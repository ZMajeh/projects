/*
 muepub.c: EPUB document handler.

 An EPUB file is a ZIP container holding an OPF package document, an
 optional NCX table of contents, and a set of XHTML chapters plus
 images. This handler reads the container with a small built in ZIP
 reader, parses the package, turns each XHTML chapter into a list of
 styled text and image blocks, paginates them into letter sized pages
 and renders them with the base 14 (Nimbus) fonts.
*/

#include "majehpdfviewer/epub.h"
#include "majehpdfviewer/pdf.h"

#include <zlib.h>

#include <string.h>
#include <stdio.h>

/* Zip signatures */
#define ZIP_LOCAL_FILE_SIG 0x04034b50
#define ZIP_CENTRAL_DIRECTORY_SIG 0x02014b50
#define ZIP_END_OF_CENTRAL_DIRECTORY_SIG 0x06054b50
#define ZIP_ENCRYPTED_FLAG 0x1

/* Page geometry (letter, like the text viewer) */
#define EPUB_PAGE_W 612
#define EPUB_PAGE_H 792
#define EPUB_MARGIN_L 64
#define EPUB_MARGIN_R 64
#define EPUB_MARGIN_T 60
#define EPUB_MARGIN_B 60
#define EPUB_CONTENT_W (EPUB_PAGE_W - EPUB_MARGIN_L - EPUB_MARGIN_R)
#define EPUB_CONTENT_H (EPUB_PAGE_H - EPUB_MARGIN_T - EPUB_MARGIN_B)

/* Body text metrics */
#define EPUB_BODY_SIZE 11
#define EPUB_BODY_LEAD 14.5f
#define EPUB_PARA_SPACE 7
#define EPUB_LIST_SPACE 2
#define EPUB_LIST_HANG 20
#define EPUB_PRE_SIZE 9.5f
#define EPUB_PRE_LEAD 12
#define EPUB_HR_SPACE 10

/* Font table indices */
enum {
	FONT_TIMES, FONT_TIMES_BOLD, FONT_TIMES_ITALIC, FONT_TIMES_BOLDITALIC,
	FONT_COURIER,
	FONT_COUNT
};

/* Block kinds */
enum {
	EPUB_B_PARA,
	EPUB_B_HEADING,
	EPUB_B_PRE,
	EPUB_B_IMAGE,
	EPUB_B_HR
};

/* Page kinds */
enum {
	EPUB_PAGE_TITLE,
	EPUB_PAGE_CONTENT
};

typedef struct epub_document_s epub_document;
typedef struct epub_page_s epub_page;
typedef struct epub_block_s epub_block;
typedef struct epub_run_s epub_run;
typedef struct epub_toc_entry_s epub_toc_entry;

/* A styled span of UTF-8 text inside a block */
struct epub_run_s
{
	char *text;
	int len;
	int bold;
	int italic;
	int mono;
	float size; /* 0 = inherit body size */
	float rise; /* superscript raise in points */
	epub_run *next;
};

/* A flow block: paragraph, heading, preformatted text, image or rule */
struct epub_block_s
{
	int kind; /* EPUB_B_* */
	int level; /* heading level 1..6 */
	int indent; /* extra left indent in points */
	int center;
	int right;
	int page_break_before;
	int spacing_after;
	char *marker; /* list marker e.g. "1. " or bullet, or NULL */
	char *image_path; /* zip path for EPUB_B_IMAGE */
	fz_image *image; /* loaded image (also held by document cache) */
	float image_w, image_h; /* natural size in points */
	int chapter; /* spine index this block came from */
	epub_run *runs;
	epub_block *next;
};

/* Positioned output produced by the layout pass */
typedef struct epub_draw_run_s
{
	int font; /* FONT_* */
	float size;
	float rise;
	float x, y; /* baseline origin in page space */
	char *text;
	int len;
} epub_draw_run;

typedef struct epub_draw_image_s
{
	fz_image *image;
	float x, y, w, h;
} epub_draw_image;

typedef struct epub_draw_rect_s
{
	float x0, y0, x1, y1;
} epub_draw_rect;

struct epub_page_s
{
	int number;
	int kind; /* EPUB_PAGE_TITLE or EPUB_PAGE_CONTENT */
	int first_run, run_count;
	int first_image, image_count;
	int first_rect, rect_count;
};

/* Manifest item */
typedef struct epub_item_s
{
	char *id;
	char *href; /* resolved zip path */
	char *type;
	char *properties;
} epub_item;

/* TOC entry (before layout we only know titles and hrefs) */
struct epub_toc_entry_s
{
	char *title;
	char *href;
	int spine_index; /* resolved spine index or -1 */
	epub_toc_entry *next;
};

/* Zip entry */
typedef struct epub_entry_s
{
	char *name;
	int offset;
	int csize, usize;
} epub_entry;

/* Cached decoded image, keyed by zip path */
typedef struct epub_image_cache_s
{
	char *path;
	fz_image *image;
} epub_image_cache;

struct epub_document_s
{
	fz_document super;
	fz_context *ctx;
	fz_stream *file;
	int entry_count;
	epub_entry *entry;

	/* package info */
	char *opf_dir; /* directory containing the OPF file */
	char *title;
	char *author;
	char *cover_path;

	epub_item *items;
	int item_count;

	char **spine; /* manifest ids in reading order */
	int spine_count;

	epub_toc_entry *toc;
	char *toc_dir; /* directory of the toc file, for resolving hrefs */

	/* fonts */
	fz_font *fonts[FONT_COUNT];

	/* image cache */
	epub_image_cache *images;
	int image_count, image_cap;

	/* parsed flow */
	epub_block *blocks; /* linked list of all blocks */
	int block_count;
	int *chapter_start_page; /* per spine index */

	/* flattened display lists */
	epub_draw_run *runs;
	int run_count, run_cap;
	epub_draw_image *draw_images;
	int draw_image_count, draw_image_cap;
	epub_draw_rect *rects;
	int rect_count, rect_cap;

	epub_page *pages;
	int page_count, page_cap;
	int has_title_page;
};

/* ---- Utility strings ---- */

static char *
epub_strdup(fz_context *ctx, const char *s)
{
	char *d;
	int n;
	if (!s)
		return NULL;
	n = strlen(s) + 1;
	d = fz_malloc(ctx, n);
	memcpy(d, s, n);
	return d;
}

static char *
epub_strndup(fz_context *ctx, const char *s, int n)
{
	char *d = fz_malloc(ctx, n + 1);
	memcpy(d, s, n);
	d[n] = 0;
	return d;
}

static int
epub_is_white(int c)
{
	return c == ' ' || c == '\t' || c == '\r' || c == '\n';
}

/* ---- ZIP reader (adapted from the CBZ handler) ---- */

static inline int epub_getshort(fz_stream *file)
{
	int a = fz_read_byte(file);
	int b = fz_read_byte(file);
	return a | b << 8;
}

static inline int epub_getlong(fz_stream *file)
{
	int a = fz_read_byte(file);
	int b = fz_read_byte(file);
	int c = fz_read_byte(file);
	int d = fz_read_byte(file);
	return a | b << 8 | c << 16 | d << 24;
}

static void *epub_zip_alloc_items(void *ctx, unsigned int items, unsigned int size)
{
	return fz_malloc_array(ctx, items, size);
}

static void epub_zip_free(void *ctx, void *ptr)
{
	fz_free(ctx, ptr);
}

static unsigned char *
epub_read_zip_entry(epub_document *doc, epub_entry *entry, int *sizep)
{
	fz_context *ctx = doc->ctx;
	fz_stream *file = doc->file;
	int sig, general, method, namelength, extralength;
	unsigned char *cdata;
	int code;

	fz_seek(file, entry->offset, 0);

	sig = epub_getlong(file);
	if (sig != ZIP_LOCAL_FILE_SIG)
		fz_throw(ctx, FZ_ERROR_GENERIC, "wrong zip local file signature (0x%x)", sig);

	(void) epub_getshort(file); /* version */
	general = epub_getshort(file); /* general purpose flags */
	if (general & ZIP_ENCRYPTED_FLAG)
		fz_throw(ctx, FZ_ERROR_GENERIC, "epub zipfile content is encrypted");

	method = epub_getshort(file);
	(void) epub_getshort(file); /* file time */
	(void) epub_getshort(file); /* file date */
	(void) epub_getlong(file); /* crc-32 */
	(void) epub_getlong(file); /* csize */
	(void) epub_getlong(file); /* usize */
	namelength = epub_getshort(file);
	extralength = epub_getshort(file);

	fz_seek(file, namelength + extralength, 1);

	cdata = fz_malloc(ctx, entry->csize ? entry->csize : 1);
	fz_try(ctx)
	{
		fz_read(file, cdata, entry->csize);
	}
	fz_catch(ctx)
	{
		fz_free(ctx, cdata);
		fz_rethrow(ctx);
	}

	if (method == 0)
	{
		*sizep = entry->usize;
		return cdata;
	}

	if (method == 8)
	{
		unsigned char *udata = fz_malloc(ctx, entry->usize ? entry->usize : 1);
		z_stream stream;

		memset(&stream, 0, sizeof stream);
		stream.zalloc = epub_zip_alloc_items;
		stream.zfree = epub_zip_free;
		stream.opaque = ctx;
		stream.next_in = cdata;
		stream.avail_in = entry->csize;
		stream.next_out = udata;
		stream.avail_out = entry->usize;

		fz_try(ctx)
		{
			code = inflateInit2(&stream, -15);
			if (code != Z_OK)
				fz_throw(ctx, FZ_ERROR_GENERIC, "zlib inflateInit2 error: %s", stream.msg);
			code = inflate(&stream, Z_FINISH);
			if (code != Z_STREAM_END)
			{
				inflateEnd(&stream);
				fz_throw(ctx, FZ_ERROR_GENERIC, "zlib inflate error: %s", stream.msg);
			}
			inflateEnd(&stream);
		}
		fz_catch(ctx)
		{
			fz_free(ctx, cdata);
			fz_free(ctx, udata);
			fz_rethrow(ctx);
		}

		fz_free(ctx, cdata);
		*sizep = entry->usize;
		return udata;
	}

	fz_free(ctx, cdata);
	fz_throw(ctx, FZ_ERROR_GENERIC, "unknown zip method: %d", method);
	return NULL;
}

static void
epub_read_zip_dir_imp(epub_document *doc, int startoffset)
{
	fz_context *ctx = doc->ctx;
	fz_stream *file = doc->file;
	int i, n, cdir_len, cdir_off;
	int sig;

	fz_seek(file, startoffset, 0);
	sig = epub_getlong(file);
	if (sig != ZIP_END_OF_CENTRAL_DIRECTORY_SIG)
		fz_throw(ctx, FZ_ERROR_GENERIC, "wrong zip end of central directory signature (0x%x)", sig);

	(void) epub_getshort(file); /* disk */
	(void) epub_getshort(file); /* start disk */
	n = epub_getshort(file); /* entries on this disk */
	(void) epub_getshort(file); /* total entries */
	cdir_len = epub_getlong(file); /* cdir length */
	cdir_off = epub_getlong(file); /* cdir offset */

	(void) cdir_len;

	fz_seek(file, cdir_off, 0);

	doc->entry = fz_malloc_array(ctx, n ? n : 1, sizeof(epub_entry));
	doc->entry_count = 0;

	for (i = 0; i < n; i++)
	{
		int namelength, extralength, commentlength;
		int method;
		unsigned int crc, csize, usize;
		int general;
		epub_entry *entry;

		sig = epub_getlong(file);
		if (sig != ZIP_CENTRAL_DIRECTORY_SIG)
			fz_throw(ctx, FZ_ERROR_GENERIC, "wrong zip central directory signature (0x%x)", sig);

		(void) epub_getshort(file); /* version made by */
		(void) epub_getshort(file); /* version to extract */
		general = epub_getshort(file);
		method = epub_getshort(file);
		(void) epub_getshort(file); /* file time */
		(void) epub_getshort(file); /* file date */
		crc = epub_getlong(file); /* crc-32 */
		csize = epub_getlong(file); /* csize */
		usize = epub_getlong(file); /* usize */
		namelength = epub_getshort(file);
		extralength = epub_getshort(file);
		commentlength = epub_getshort(file);
		(void) epub_getshort(file); /* disk number */
		(void) epub_getshort(file); /* internal attributes */
		(void) epub_getlong(file); /* external attributes */

		entry = &doc->entry[doc->entry_count];
		entry->offset = epub_getlong(file); /* local header offset */
		entry->name = fz_malloc(ctx, namelength + 1);
		fz_read(file, (unsigned char *)entry->name, namelength);
		entry->name[namelength] = 0;
		entry->csize = csize;
		entry->usize = usize;

		(void) general;
		(void) method;
		(void) crc;

		fz_seek(file, extralength + commentlength, 1);

		doc->entry_count++;
	}
}

static void
epub_read_zip_dir(epub_document *doc)
{
	fz_context *ctx = doc->ctx;
	fz_stream *file = doc->file;
	int try, i;

	fz_seek(file, 0, 2);
	try = (int)fz_tell(file);
	if (try > 65536)
		try = 65536;

	for (i = try; i >= 0; i--)
	{
		fz_seek(file, i, 0);
		if (epub_getlong(file) == ZIP_END_OF_CENTRAL_DIRECTORY_SIG)
		{
			epub_read_zip_dir_imp(doc, i);
			return;
		}
	}

	fz_throw(ctx, FZ_ERROR_GENERIC, "cannot find zip end of central directory record");
}

static epub_entry *
epub_find_entry(epub_document *doc, const char *name)
{
	int i;
	for (i = 0; i < doc->entry_count; i++)
		if (!strcmp(doc->entry[i].name, name))
			return &doc->entry[i];
	return NULL;
}

/* Read a file from the zip; returns NUL terminated data, len set.
 fz_free the result when done. Throws if the entry is missing. */
static unsigned char *
epub_read_file(epub_document *doc, const char *name, int *lenp)
{
	epub_entry *entry = epub_find_entry(doc, name);
	unsigned char *data;
	int size;

	if (!entry)
		fz_throw(doc->ctx, FZ_ERROR_GENERIC, "cannot find file in epub: %s", name);

	data = epub_read_zip_entry(doc, entry, &size);
	data = fz_resize_array(doc->ctx, data, size + 1, 1);
	data[size] = 0; /* ensure NUL termination for the xml parser */
	*lenp = size;
	return data;
}

/* ---- Path resolution ---- */

static void epub_percent_decode(char *s);

/* Resolve a relative href against a base directory inside the zip.
 Strips fragments and queries. Returns a fz_malloc'd string. */
static char *
epub_resolve_path(fz_context *ctx, const char *dir, const char *href)
{
	char *path, *out;
	const char *s, *seg, *stop;
	int len;

	/* strip fragment and query */
	len = strlen(href);
	while (len > 0)
	{
		if (href[len-1] == '#' || href[len-1] == '?')
		{
			len--;
			continue;
		}
		break;
	}

	path = epub_strndup(ctx, href, len);
	epub_percent_decode(path);

	if (dir && dir[0])
	{
		char *joined = fz_malloc(ctx, strlen(dir) + strlen(path) + 2);
		sprintf(joined, "%s/%s", dir, path);
		fz_free(ctx, path);
		path = joined;
	}

	/* normalize ./ and ../ segments */
	out = fz_malloc(ctx, strlen(path) + 2);
	out[0] = 0;
	s = path;
	while (*s)
	{
		if (*s == '/') { s++; continue; }
		seg = s;
		while (*s && *s != '/') s++;
		stop = s;
		if (stop - seg == 1 && seg[0] == '.')
			continue;
		if (stop - seg == 2 && seg[0] == '.' && seg[1] == '.')
		{
			char *slash = strrchr(out, '/');
			if (slash) *slash = 0;
			else out[0] = 0;
			continue;
		}
		if (out[0]) strcat(out, "/");
		strncat(out, seg, stop - seg);
	}

	fz_free(ctx, path);
	return out;
}

/* Return the directory part of a zip path ("a/b/c.xhtml" -> "a/b"). */
static char *
epub_dirname(fz_context *ctx, const char *path)
{
	const char *slash = strrchr(path, '/');
	if (!slash)
		return epub_strdup(ctx, "");
	return epub_strndup(ctx, path, slash - path);
}

/* ---- XML helpers ---- */

static fz_xml *
epub_xml_root(fz_xml *xml)
{
	while (xml && !fz_xml_tag(xml)[0])
		xml = fz_xml_next(xml);
	return xml;
}

static fz_xml *
epub_find_child(fz_xml *parent, const char *tag)
{
	fz_xml *n;
	for (n = fz_xml_down(parent); n; n = fz_xml_next(n))
	{
		char *t = fz_xml_tag(n);
		char *c = strrchr(t, ':');
		if (!strcmp(c ? c + 1 : t, tag))
			return n;
	}
	return NULL;
}

/* Strip an XML namespace prefix: "opf:package" -> "package". */
static const char *
epub_bare_tag(fz_xml *node)
{
	char *tag = fz_xml_tag(node);
	char *c = strrchr(tag, ':');
	return c ? c + 1 : tag;
}

static int
epub_hexval(int c)
{
	if (c >= '0' && c <= '9') return c - '0';
	if (c >= 'a' && c <= 'f') return c - 'a' + 10;
	if (c >= 'A' && c <= 'F') return c - 'A' + 10;
	return -1;
}

/* Decode %XX sequences in place ("My%20Book.xhtml" -> "My Book.xhtml"). */
static void
epub_percent_decode(char *s)
{
	char *d = s;
	while (*s)
	{
		int hi, lo;
		if (s[0] == '%' && (hi = epub_hexval(s[1])) >= 0 && (lo = epub_hexval(s[2])) >= 0)
		{
			*d++ = (hi << 4) | lo;
			s += 3;
		}
		else
			*d++ = *s++;
	}
	*d = 0;
}

/* Duplicate the value of an attribute, or return NULL. */
static char *
fz_xml_att_dup(fz_context *ctx, fz_xml *node, const char *att)
{
	char *v = fz_xml_att(node, att);
	return v ? epub_strdup(ctx, v) : NULL;
}

static int
epub_strstr(const char *hay, const char *needle)
{
	const char *p, *q;
	if (!hay || !needle || !needle[0])
		return 0;
	for (p = hay; *p; p++)
	{
		for (q = needle; *q && *p == *q; q++, p++)
			;
		if (!*q) return 1;
		p -= (q - needle);
	}
	return 0;
}

/* ---- Text measurement ---- */

static float
epub_glyph_width(fz_context *ctx, fz_font *font, int gid, float size)
{
	float w = fz_advance_glyph(ctx, font, gid) * size;
	if (w > 100) w /= 1000.0f; /* Handle EM units vs pre-scaled */
	return w;
}

static float
epub_text_width(fz_context *ctx, fz_font *font, const char *s, int len, float size)
{
	float w = 0;
	int ucs, gid;
	const char *p = s;
	const char *end = s + len;
	while (p < end)
	{
		p += fz_chartorune(&ucs, p);
		gid = fz_encode_character(ctx, font, ucs);
		w += epub_glyph_width(ctx, font, gid, size);
	}
	return w;
}

static int
epub_run_font(const epub_run *run)
{
	if (run->mono) return FONT_COURIER;
	if (run->bold && run->italic) return FONT_TIMES_BOLDITALIC;
	if (run->bold) return FONT_TIMES_BOLD;
	if (run->italic) return FONT_TIMES_ITALIC;
	return FONT_TIMES;
}

/* ---- Package parsing ---- */

static epub_item *
epub_find_item(epub_document *doc, const char *id)
{
	int i;
	if (!id) return NULL;
	for (i = 0; i < doc->item_count; i++)
		if (doc->items[i].id && !strcmp(doc->items[i].id, id))
			return &doc->items[i];
	return NULL;
}

static char *
epub_find_item_by_type(epub_document *doc, const char *type)
{
	int i;
	for (i = 0; i < doc->item_count; i++)
		if (doc->items[i].type && !strcmp(doc->items[i].type, type))
			return doc->items[i].href;
	return NULL;
}

static void epub_parse_ncx_toc(epub_document *doc, char *path);
static void epub_parse_nav_toc(epub_document *doc, char *path);

static void
epub_parse_opf(epub_document *doc, unsigned char *data, int len)
{
	fz_context *ctx = doc->ctx;
	fz_xml *root, *node, *item;
	char *cover_id = NULL;

	root = epub_xml_root(fz_parse_xml(ctx, data, len + 1));
	if (!root || strcmp(epub_bare_tag(root), "package"))
	{
		fz_free_xml(ctx, root);
		fz_throw(ctx, FZ_ERROR_GENERIC, "epub opf file is not a package document");
	}

	fz_try(ctx)
	{
		/* metadata: dc:title, dc:creator */
		node = epub_find_child(root, "metadata");
		if (node)
		{
			fz_xml *n;
			for (n = fz_xml_down(node); n; n = fz_xml_next(n))
			{
				char *tag = (char *)epub_bare_tag(n);
				char *text = fz_xml_text(n);
				if (text)
				{
					char *end;
					while (*text && epub_is_white(*text)) text++;
					end = text + strlen(text);
					while (end > text && epub_is_white(end[-1])) *--end = 0;
					if (*text)
					{
						char *bare = strrchr(tag, ':');
						bare = bare ? bare + 1 : tag;
						if (!strcmp(bare, "title") && !doc->title)
							doc->title = epub_strdup(ctx, text);
						else if (!strcmp(bare, "creator") && !doc->author)
							doc->author = epub_strdup(ctx, text);
					}
				}
				if (!strcmp(tag, "meta"))
				{
					char *name = fz_xml_att(n, "name");
					char *content = fz_xml_att(n, "content");
					if (name && content && !strcmp(name, "cover"))
						cover_id = epub_strdup(ctx, content);
				}
			}
		}

		/* manifest */
		node = epub_find_child(root, "manifest");
		if (node)
		{
			int cap = 16;
			doc->items = fz_malloc_array(ctx, cap, sizeof(epub_item));
			for (item = fz_xml_down(node); item; item = fz_xml_next(item))
			{
				char *id, *href, *type, *props;
				if (strcmp(epub_bare_tag(item), "item"))
					continue;
				id = fz_xml_att(item, "id");
				href = fz_xml_att(item, "href");
				type = fz_xml_att(item, "media-type");
				props = fz_xml_att(item, "properties");
				if (!id || !href)
					continue;
				if (doc->item_count == cap)
				{
					cap *= 2;
					doc->items = fz_resize_array(ctx, doc->items, cap, sizeof(epub_item));
				}
				doc->items[doc->item_count].id = epub_strdup(ctx, id);
				doc->items[doc->item_count].href = epub_resolve_path(ctx, doc->opf_dir, href);
				doc->items[doc->item_count].type = type ? epub_strdup(ctx, type) : NULL;
				doc->items[doc->item_count].properties = props ? epub_strdup(ctx, props) : NULL;
				doc->item_count++;
			}
		}

		/* spine */
		node = epub_find_child(root, "spine");
		if (node)
		{
			fz_xml *itemref;
			int cap = 16;
			doc->spine = fz_malloc_array(ctx, cap, sizeof(char *));
			for (itemref = fz_xml_down(node); itemref; itemref = fz_xml_next(itemref))
			{
				char *idref, *linear;
				if (strcmp(epub_bare_tag(itemref), "itemref"))
					continue;
				linear = fz_xml_att(itemref, "linear");
				if (linear && !strcmp(linear, "no"))
					continue;
				idref = fz_xml_att(itemref, "idref");
				if (!idref)
					continue;
				if (doc->spine_count == cap)
				{
					cap *= 2;
					doc->spine = fz_resize_array(ctx, doc->spine, cap, sizeof(char *));
				}
				doc->spine[doc->spine_count++] = epub_strdup(ctx, idref);
			}
		}

		/* cover image */
		if (cover_id)
		{
			epub_item *cov = epub_find_item(doc, cover_id);
			if (cov)
				doc->cover_path = epub_strdup(ctx, cov->href);
		}
		if (!doc->cover_path)
		{
			int i;
			for (i = 0; i < doc->item_count; i++)
			{
				epub_item *it = &doc->items[i];
				if (it->properties && epub_strstr(it->properties, "cover-image"))
				{
					doc->cover_path = epub_strdup(ctx, it->href);
					break;
				}
			}
		}
		if (!doc->cover_path)
		{
			int i;
			for (i = 0; i < doc->item_count; i++)
			{
				epub_item *it = &doc->items[i];
				if (it->id && !strcmp(it->id, "cover") && it->type &&
					(!strcmp(it->type, "image/jpeg") || !strcmp(it->type, "image/png")))
				{
					doc->cover_path = epub_strdup(ctx, it->href);
					break;
				}
			}
		}

		/* nav.xhtml (EPUB3) TOC */
		{
			int i;
			for (i = 0; i < doc->item_count; i++)
			{
				epub_item *it = &doc->items[i];
				if (it->properties && epub_strstr(it->properties, "nav"))
					epub_parse_nav_toc(doc, it->href);
			}
		}

		/* NCX TOC (EPUB2) */
		{
			char *ncx = epub_find_item_by_type(doc, "application/x-dtbncx+xml");
			if (ncx)
				epub_parse_ncx_toc(doc, ncx);
		}
	}
	fz_catch(ctx)
	{
		fz_free(ctx, cover_id);
		fz_free_xml(ctx, root);
		fz_rethrow(ctx);
	}

	fz_free(ctx, cover_id);
	fz_free_xml(ctx, root);
}

/* ---- Table of contents ---- */

static void
epub_toc_append(epub_document *doc, char *title, char *href)
{
	epub_toc_entry *entry, **tail;

	entry = fz_malloc_struct(doc->ctx, epub_toc_entry);
	entry->title = title;
	entry->href = href;
	entry->spine_index = -1;

	/* resolve href relative to the toc file's directory */
	if (doc->toc_dir)
	{
		char *resolved = epub_resolve_path(doc->ctx, doc->toc_dir, href);
		fz_free(doc->ctx, href);
		entry->href = resolved;
	}

	for (tail = &doc->toc; *tail; tail = &(*tail)->next)
		;
	*tail = entry;
}

/* Concatenate all descendant text of an element into a fz_malloc'd string. */
static char *
epub_xml_text_content(fz_context *ctx, fz_xml *node)
{
	fz_xml *n;
	char *out = NULL;
	int len = 0, cap = 0;

	for (n = fz_xml_down(node); n; n = fz_xml_next(n))
	{
		char *t = fz_xml_text(n);
		if (t)
		{
			int tl = strlen(t);
			if (len + tl + 1 > cap)
			{
				cap = len + tl + 64;
				out = fz_resize_array(ctx, out, cap, 1);
			}
			memcpy(out + len, t, tl);
			len += tl;
			out[len] = 0;
		}
		else
		{
			/* recurse into nested elements */
			char *sub = epub_xml_text_content(ctx, n);
			if (sub)
			{
				int sl = strlen(sub);
				if (len + sl + 2 > cap)
				{
					cap = len + sl + 64;
					out = fz_resize_array(ctx, out, cap, 1);
				}
				if (len > 0 && out[len-1] != ' ')
					out[len++] = ' ';
				memcpy(out + len, sub, sl);
				len += sl;
				out[len] = 0;
				fz_free(ctx, sub);
			}
		}
	}
	return out;
}

/* Trim leading/trailing whitespace of a NUL-terminated string in place. */
static char *
epub_trim(char *s)
{
	char *end;
	while (*s && epub_is_white(*s)) s++;
	if (*s == 0) return s;
	end = s + strlen(s);
	while (end > s && epub_is_white(end[-1])) end--;
	*end = 0;
	return s;
}

static void
epub_parse_ncx_navpoints(epub_document *doc, fz_xml *node, int depth)
{
	fz_context *ctx = doc->ctx;
	fz_xml *n;

	if (depth > 6)
		return;

	for (n = fz_xml_down(node); n; n = fz_xml_next(n))
	{
		if (strcmp(epub_bare_tag(n), "navPoint"))
			continue;

		{
			fz_xml *label = epub_find_child(n, "navLabel");
			fz_xml *content = epub_find_child(n, "content");
			char *title = NULL;
			char *src = NULL;

			if (label)
			{
				fz_xml *t = epub_find_child(label, "text");
				if (t && fz_xml_text(t))
					title = epub_strdup(ctx, fz_xml_text(t));
			}
			if (content)
				src = fz_xml_att_dup(ctx, content, "src");

			if (title && src)
				epub_toc_append(doc, title, src);
			else
			{
				fz_free(ctx, title);
				fz_free(ctx, src);
			}
		}

		epub_parse_ncx_navpoints(doc, n, depth + 1);
	}
}

static void
epub_parse_ncx_toc(epub_document *doc, char *path)
{
	fz_context *ctx = doc->ctx;
	unsigned char *data;
	fz_xml *root, *map;
	int len;

	fz_var(data);

	fz_try(ctx)
	{
		data = epub_read_file(doc, path, &len);
		doc->toc_dir = epub_dirname(ctx, path);
		root = epub_xml_root(fz_parse_xml(ctx, data, len + 1));
		if (root && !strcmp(epub_bare_tag(root), "ncx"))
		{
			map = epub_find_child(root, "navMap");
			if (map)
				epub_parse_ncx_navpoints(doc, map, 0);
		}
		fz_free_xml(ctx, root);
	}
	fz_always(ctx)
	{
		fz_free(ctx, data);
	}
	fz_catch(ctx)
	{
		fz_warn(ctx, "cannot parse epub ncx: %s", path);
	}
}

static void
epub_parse_nav_toc(epub_document *doc, char *path)
{
	fz_context *ctx = doc->ctx;
	unsigned char *data;
	fz_xml *root, *nav, *n;
	int len;

	fz_var(data);

	fz_try(ctx)
	{
		data = epub_read_file(doc, path, &len);
		doc->toc_dir = epub_dirname(ctx, path);
		root = epub_xml_root(fz_parse_xml(ctx, data, len + 1));

		/* find the <nav epub:type="toc"> element (or first nav) */
		nav = NULL;
		if (root)
		{
			for (n = root; n; n = fz_xml_next(n))
			{
				char *epubtype = fz_xml_att(n, "epub:type");
				if (!strcmp(epub_bare_tag(n), "nav"))
				{
					if (!nav)
						nav = n;
					if (epubtype && !strcmp(epubtype, "toc"))
					{
						nav = n;
						break;
					}
				}
			}
		}

		if (nav)
		{
			/* walk list items: <li><a href>text</a><ul>...</ul></li> */
			for (n = fz_xml_down(nav); n; n = fz_xml_next(n))
			{
				fz_xml *li;
				for (li = fz_xml_down(n); li; li = fz_xml_next(li))
				{
					fz_xml *a;
					if (strcmp(fz_xml_tag(li), "li") && strcmp(fz_xml_tag(li), "ol") && strcmp(fz_xml_tag(li), "ul"))
						continue;
					if (!strcmp(fz_xml_tag(li), "li"))
					{
						for (a = fz_xml_down(li); a; a = fz_xml_next(a))
						{
							if (!strcmp(fz_xml_tag(a), "a"))
							{
								char *href = fz_xml_att_dup(ctx, a, "href");
								char *text = fz_xml_text(a);
								if (href)
									epub_toc_append(doc, text ? epub_strdup(ctx, text) : NULL, href);
							}
							else if (!strcmp(fz_xml_tag(a), "ol") || !strcmp(fz_xml_tag(a), "ul"))
							{
								fz_xml *sub;
								for (sub = fz_xml_down(a); sub; sub = fz_xml_next(sub))
								{
									fz_xml *sa;
									if (strcmp(fz_xml_tag(sub), "li"))
										continue;
									for (sa = fz_xml_down(sub); sa; sa = fz_xml_next(sa))
									{
										if (!strcmp(fz_xml_tag(sa), "a"))
										{
											char *href = fz_xml_att_dup(ctx, sa, "href");
											char *text = fz_xml_text(sa);
											if (href)
												epub_toc_append(doc, text ? epub_strdup(ctx, text) : NULL, href);
										}
									}
								}
							}
						}
					}
				}
			}
		}

		fz_free_xml(ctx, root);
	}
	fz_always(ctx)
	{
		fz_free(ctx, data);
	}
	fz_catch(ctx)
	{
		fz_warn(ctx, "cannot parse epub nav: %s", path);
	}
}

/* ---- Chapter parsing: XHTML -> blocks of styled runs ---- */

typedef struct epub_list_state_s
{
	int ordered;
	int count;
} epub_list_state;

typedef struct epub_parser_s
{
	epub_document *doc;
	char *dir; /* directory of the chapter file */
	int chapter;

	/* current inline style */
	int bold, italic, mono;
	float size; /* 0 = body size */
	float rise;

	/* current block settings */
	int heading; /* 0 or heading level */
	int pre;
	int center;
	int right;
	int pending_break;

	/* current block under construction */
	epub_block *block;

	/* list and quote nesting */
	epub_list_state list[32];
	int list_depth;
	int quote_depth;
	int dd_indent;

	int suppress; /* inside script/style/head */
} epub_parser;

/* Decode a named entity into UTF-8 at d; returns bytes written (0 to drop). */
static int
epub_entity(const char *name, int len, char *d)
{
	struct { const char *name; const char *utf8; } table[] = {
		{ "nbsp", "\302\240" }, { "shy", "" },
		{ "mdash", "\342\200\224" }, { "ndash", "\342\200\223" },
		{ "hellip", "\342\200\246" }, { "middot", "\302\267" },
		{ "lsquo", "\342\200\230" }, { "rsquo", "\342\200\231" },
		{ "ldquo", "\342\200\234" }, { "rdquo", "\342\200\235" },
		{ "laquo", "\302\253" }, { "raquo", "\302\273" },
		{ "copy", "\302\251" }, { "reg", "\302\256" },
		{ "trade", "\342\204\242" }, { "deg", "\302\260" },
		{ "plusmn", "\302\261" }, { "times", "\303\227" },
		{ "divide", "\303\267" }, { "frac12", "\302\275" },
		{ NULL, NULL }
	};
	int i;
	/* name comes without the & but may include the trailing ';' */
	if (len > 0 && name[len-1] == ';') len--;
	for (i = 0; table[i].name; i++)
	{
		if ((int)strlen(table[i].name) == len && !strncmp(table[i].name, name, len))
		{
			int n = strlen(table[i].utf8);
			memcpy(d, table[i].utf8, n);
			return n;
		}
	}
	return 0;
}

/* Copy text into a buffer, decoding leftover named entities
 (the xml parser only decodes &amp; &lt; &gt; &quot; &apos; and &#..;) */
static char *
epub_decode_text(fz_context *ctx, const char *s, int len, int *outlen)
{
	char *d = fz_malloc(ctx, len * 2 + 2);
	char *o = d;
	const char *end = s + len;

	while (s < end)
	{
		if (*s == '&')
		{
			const char *e = s + 1;
			while (e < end && *e != ';' && (e - s) < 12) e++;
			if (e < end && *e == ';' && epub_entity(s + 1, e - s - 1, o))
			{
				o += strlen(o);
				s = e + 1;
				continue;
			}
			/* unknown entity: drop it */
			s = (e < end && *e == ';') ? e + 1 : s + 1;
			continue;
		}
		*o++ = *s++;
	}
	*o = 0;
	*outlen = o - d;
	return d;
}

static void
epub_block_append(epub_document *doc, epub_block *block)
{
	epub_block **tail = &doc->blocks;
	block->next = NULL;
	while (*tail) tail = &(*tail)->next;
	*tail = block;
	doc->block_count++;
}

static void
epub_end_block(epub_parser *p)
{
	epub_document *doc = p->doc;
	epub_block *b = p->block;
	if (!b) return;
	p->block = NULL;
	if (b->runs == NULL && b->kind != EPUB_B_IMAGE && b->kind != EPUB_B_HR)
	{
		/* empty block: drop it */
		epub_run *run = b->runs;
		while (run)
		{
			epub_run *next = run->next;
			fz_free(doc->ctx, run->text);
			fz_free(doc->ctx, run);
			run = next;
		}
		fz_free(doc->ctx, b->marker);
		fz_free(doc->ctx, b->image_path);
		fz_free(doc->ctx, b);
		return;
	}
	epub_block_append(doc, b);
}

static void
epub_start_block(epub_parser *p, int kind, int level)
{
	epub_document *doc = p->doc;
	epub_block *b;

	epub_end_block(p);

	b = fz_malloc_struct(doc->ctx, epub_block);
	b->kind = kind;
	b->level = level;
	b->chapter = p->chapter;
	b->indent = (p->quote_depth + p->list_depth + p->dd_indent) * 22;
	b->center = p->center;
	b->right = p->right;
	if (p->pending_break)
	{
		b->page_break_before = 1;
		p->pending_break = 0;
	}
	b->spacing_after = (p->list_depth > 0) ? EPUB_LIST_SPACE : EPUB_PARA_SPACE;
	p->block = b;
}

/* Append a styled run; merges with the previous run when the style matches. */
static void
epub_add_run(epub_parser *p, const char *text, int len)
{
	epub_document *doc = p->doc;
	epub_run *run;

	if (len <= 0) return;
	if (!p->block) epub_start_block(p, EPUB_B_PARA, 0);
	if (!p->block) return;

	run = p->block->runs;
	if (run)
	{
		while (run->next) run = run->next;
		if (run->bold == p->bold && run->italic == p->italic &&
			run->mono == p->mono && run->size == p->size && run->rise == p->rise)
		{
			char *nt = fz_malloc(doc->ctx, run->len + len + 1);
			memcpy(nt, run->text, run->len);
			memcpy(nt + run->len, text, len);
			nt[run->len + len] = 0;
			fz_free(doc->ctx, run->text);
			run->text = nt;
			run->len += len;
			return;
		}
	}

	run = fz_malloc_struct(doc->ctx, epub_run);
	run->text = epub_strndup(doc->ctx, text, len);
	run->len = len;
	run->bold = p->bold;
	run->italic = p->italic;
	run->mono = p->mono;
	run->size = p->size;
	run->rise = p->rise;

	if (!p->block->runs)
		p->block->runs = run;
	else
	{
		epub_run *last = p->block->runs;
		while (last->next) last = last->next;
		last->next = run;
	}
}

/* Add text with whitespace collapsing (or verbatim in pre mode). */
static void
epub_add_text(epub_parser *p, const char *s, int len)
{
	char *decoded;
	int dlen;
	int i, start;

	if (p->suppress || len <= 0)
		return;

	decoded = epub_decode_text(p->doc->ctx, s, len, &dlen);
	fz_try(p->doc->ctx)
	{
		if (p->pre)
		{
			epub_add_run(p, decoded, dlen);
		}
		else
		{
			/* collapse whitespace runs into single spaces */
			i = 0;
			while (i < dlen)
			{
				if (epub_is_white(decoded[i]))
				{
					/* emit a single space (unless at block start) */
					while (i < dlen && epub_is_white(decoded[i])) i++;
					if (p->block && p->block->runs)
					{
						char sp = ' ';
						epub_add_run(p, &sp, 1);
					}
				}
				else
				{
					start = i;
					while (i < dlen && !epub_is_white(decoded[i])) i++;
					epub_add_run(p, decoded + start, i - start);
				}
			}
		}
	}
	fz_always(p->doc->ctx)
	{
		fz_free(p->doc->ctx, decoded);
	}
	fz_catch(p->doc->ctx)
	{
		fz_rethrow(p->doc->ctx);
	}
}

/* Look up (and decode) an image by zip path; cached on the document. */
static fz_image *
epub_load_image(epub_document *doc, const char *path)
{
	fz_context *ctx = doc->ctx;
	epub_entry *entry;
	unsigned char *data;
	fz_image *image;
	int size, i;

	for (i = 0; i < doc->image_count; i++)
		if (!strcmp(doc->images[i].path, path))
			return doc->images[i].image;

	entry = epub_find_entry(doc, path);
	if (!entry)
		return NULL;

	/* only try to decode formats the engine supports */
	if (!(strstr(path, ".jpg") || strstr(path, ".jpeg") || strstr(path, ".png") ||
		strstr(path, ".JPG") || strstr(path, ".JPEG") || strstr(path, ".PNG")))
		return NULL;

	data = epub_read_zip_entry(doc, entry, &size);

	image = NULL;
	fz_try(ctx)
	{
		image = fz_new_image_from_data(ctx, data, size);
		data = NULL; /* image owns data now */
	}
	fz_catch(ctx)
	{
		fz_warn(ctx, "cannot decode epub image: %s", path);
		image = NULL;
	}
	fz_free(ctx, data); /* no-op if consumed */

	if (!image)
		return NULL;

	if (doc->image_count == doc->image_cap)
	{
		doc->image_cap = doc->image_cap ? doc->image_cap * 2 : 8;
		doc->images = fz_resize_array(ctx, doc->images, doc->image_cap, sizeof(epub_image_cache));
	}
	doc->images[doc->image_count].path = epub_strdup(ctx, path);
	doc->images[doc->image_count].image = image;
	doc->image_count++;
	return image;
}

/* Handle an <img> element. */
static void
epub_parse_img(epub_parser *p, fz_xml *node)
{
	epub_document *doc = p->doc;
	char *src = fz_xml_att(node, "src");
	char *path;
	fz_image *image;

	if (!src || p->suppress)
		return;

	path = epub_resolve_path(doc->ctx, p->dir, src);
	fz_try(doc->ctx)
	{
		image = epub_load_image(doc, path);
		if (image)
		{
			epub_end_block(p);
			p->block = fz_malloc_struct(doc->ctx, epub_block);
			p->block->kind = EPUB_B_IMAGE;
			p->block->chapter = p->chapter;
			p->block->indent = (p->quote_depth + p->list_depth + p->dd_indent) * 22;
			p->block->image_path = path;
			p->block->image = image;
			p->block->image_w = image->w;
			p->block->image_h = image->h;
			if (p->pending_break)
			{
				p->block->page_break_before = 1;
				p->pending_break = 0;
			}
			p->block->spacing_after = EPUB_PARA_SPACE;
			path = NULL;
			epub_end_block(p);
		}
	}
	fz_always(doc->ctx)
	{
		fz_free(doc->ctx, path);
	}
	fz_catch(doc->ctx)
	{
		fz_rethrow(doc->ctx);
	}
}

/* Walk an XML node list building blocks and runs. */
static void epub_walk(epub_parser *p, fz_xml *node);

static void
epub_walk_children(epub_parser *p, fz_xml *node)
{
	fz_xml *child = fz_xml_down(node);
	if (child)
		epub_walk(p, child);
}

/* Recurse with a saved style snapshot. */
static void
epub_walk_styled(epub_parser *p, fz_xml *node,
	int bold, int italic, int mono, float size, float rise)
{
	int ob = p->bold, oi = p->italic, om = p->mono;
	float os = p->size, or = p->rise;

	p->bold = bold; p->italic = italic; p->mono = mono;
	p->size = size; p->rise = rise;

	epub_walk_children(p, node);

	p->bold = ob; p->italic = oi; p->mono = om;
	p->size = os; p->rise = or;
}

static void
epub_walk(epub_parser *p, fz_xml *node)
{
	fz_context *ctx = p->doc->ctx;

	for (; node; node = fz_xml_next(node))
	{
		char *tag = fz_xml_tag(node);
		char *text = fz_xml_text(node);

		/* text node */
		if (!tag[0])
		{
			if (text)
				epub_add_text(p, text, strlen(text));
			continue;
		}

		/* void / self-contained elements */
		if (!strcmp(tag, "br"))
		{
			if (!p->suppress)
			{
				char sp = ' ';
				epub_add_run(p, &sp, 1);
			}
			continue;
		}
		if (!strcmp(tag, "img"))
		{
			epub_parse_img(p, node);
			continue;
		}
		if (!strcmp(tag, "hr"))
		{
			if (!p->suppress)
			{
				char *cls = fz_xml_att(node, "class");
				if (cls && (epub_strstr(cls, "pagebreak") || epub_strstr(cls, "page-break")))
					p->pending_break = 1;
				else
				{
					epub_end_block(p);
					p->block = fz_malloc_struct(ctx, epub_block);
					p->block->kind = EPUB_B_HR;
					p->block->chapter = p->chapter;
					if (p->pending_break) { p->block->page_break_before = 1; p->pending_break = 0; }
					epub_end_block(p);
				}
			}
			continue;
		}

		/* skip non-content elements entirely */
		if (!strcmp(tag, "head") || !strcmp(tag, "script") || !strcmp(tag, "style") ||
			!strcmp(tag, "meta") || !strcmp(tag, "link") || !strcmp(tag, "title"))
		{
			continue;
		}

		/* headings */
		if (tag[0] == 'h' && tag[1] >= '1' && tag[1] <= '6' && tag[2] == 0)
		{
			int level = tag[1] - '0';
			char *cls = fz_xml_att(node, "class");
			if (!p->suppress)
			{
				if (cls && (epub_strstr(cls, "pagebreak") || epub_strstr(cls, "page-break")))
					p->pending_break = 1;
				epub_end_block(p);
				p->block = fz_malloc_struct(ctx, epub_block);
				p->block->kind = EPUB_B_HEADING;
				p->block->level = level;
				p->block->chapter = p->chapter;
				p->block->center = p->center;
				p->block->right = p->right;
				if (p->pending_break) { p->block->page_break_before = 1; p->pending_break = 0; }
				p->block->spacing_after = EPUB_PARA_SPACE;
				p->bold = 1;
				epub_walk_children(p, node);
				p->bold = 0;
				epub_end_block(p);
			}
			continue;
		}

		/* block-level containers */
		if (!strcmp(tag, "p") || !strcmp(tag, "div") || !strcmp(tag, "section") ||
			!strcmp(tag, "article") || !strcmp(tag, "aside") || !strcmp(tag, "header") ||
			!strcmp(tag, "footer") || !strcmp(tag, "figure") || !strcmp(tag, "figcaption") ||
			!strcmp(tag, "address") || !strcmp(tag, "dt") || !strcmp(tag, "dd") ||
			!strcmp(tag, "blockquote") || !strcmp(tag, "center"))
		{
			if (p->suppress) continue;
			epub_end_block(p);
			if (!strcmp(tag, "blockquote"))
				p->quote_depth++;
			else if (!strcmp(tag, "center"))
				p->center = 1;
			else if (!strcmp(tag, "dd"))
				p->dd_indent++;

			epub_walk_children(p, node);

			if (!strcmp(tag, "blockquote"))
				p->quote_depth--;
			else if (!strcmp(tag, "center"))
				p->center = 0;
			else if (!strcmp(tag, "dd"))
				p->dd_indent--;
			epub_end_block(p);
			continue;
		}

		/* lists */
		if (!strcmp(tag, "ul") || !strcmp(tag, "ol"))
		{
			if (p->suppress) continue;
			epub_end_block(p);
			if (p->list_depth < 32)
			{
				p->list[p->list_depth].ordered = !strcmp(tag, "ol");
				p->list[p->list_depth].count = 1;
				p->list_depth++;
			}
			epub_walk_children(p, node);
			if (p->list_depth > 0) p->list_depth--;
			epub_end_block(p);
			continue;
		}
		if (!strcmp(tag, "li"))
		{
			if (p->suppress) continue;
			epub_end_block(p);
			if (p->list_depth > 0 && p->list_depth <= 32)
			{
				epub_list_state *ls = &p->list[p->list_depth - 1];
				char buf[16];
				if (ls->ordered)
					sprintf(buf, "%d. ", ls->count++);
				else
					strcpy(buf, "\xe2\x80\xa2 "); /* bullet */
				p->block = fz_malloc_struct(ctx, epub_block);
				p->block->kind = EPUB_B_PARA;
				p->block->chapter = p->chapter;
				p->block->indent = (p->quote_depth + p->list_depth + p->dd_indent) * 22;
				p->block->marker = epub_strdup(ctx, buf);
				p->block->spacing_after = EPUB_LIST_SPACE;
			}
			epub_walk_children(p, node);
			epub_end_block(p);
			continue;
		}

		/* preformatted */
		if (!strcmp(tag, "pre"))
		{
			if (p->suppress) continue;
			epub_end_block(p);
			p->block = fz_malloc_struct(ctx, epub_block);
			p->block->kind = EPUB_B_PRE;
			p->block->chapter = p->chapter;
			p->block->indent = (p->quote_depth + p->list_depth + p->dd_indent) * 22;
			p->pre = 1;
			p->mono = 1;
			epub_walk_children(p, node);
			p->pre = 0;
			p->mono = 0;
			epub_end_block(p);
			continue;
		}

		/* inline style tags */
		if (!strcmp(tag, "b") || !strcmp(tag, "strong"))
		{
			epub_walk_styled(p, node, 1, p->italic, p->mono, p->size, p->rise);
			continue;
		}
		if (!strcmp(tag, "i") || !strcmp(tag, "em") || !strcmp(tag, "cite") ||
			!strcmp(tag, "var"))
		{
			epub_walk_styled(p, node, p->bold, 1, p->mono, p->size, p->rise);
			continue;
		}
		if (!strcmp(tag, "code") || !strcmp(tag, "kbd") || !strcmp(tag, "samp") ||
			!strcmp(tag, "tt"))
		{
			epub_walk_styled(p, node, p->bold, p->italic, 1, p->size, p->rise);
			continue;
		}
		if (!strcmp(tag, "small"))
		{
			float sz = p->size > 0 ? p->size - 1 : EPUB_BODY_SIZE - 1;
			epub_walk_styled(p, node, p->bold, p->italic, p->mono, sz, p->rise);
			continue;
		}
		if (!strcmp(tag, "sup"))
		{
			epub_walk_styled(p, node, p->bold, p->italic, p->mono, p->size, p->rise + 4);
			continue;
		}
		if (!strcmp(tag, "sub"))
		{
			epub_walk_styled(p, node, p->bold, p->italic, p->mono, p->size, p->rise - 3);
			continue;
		}

		/* default: recurse transparently */
		epub_walk_children(p, node);
	}
}

/* Parse one XHTML chapter file into blocks. */
static void
epub_parse_chapter(epub_document *doc, int chapter, const char *path)
{
	fz_context *ctx = doc->ctx;
	epub_parser p;
	unsigned char *data = NULL;
	fz_xml *root = NULL, *body, *n;
	int len;

	memset(&p, 0, sizeof p);
	p.doc = doc;
	p.chapter = chapter;
	p.dir = epub_dirname(ctx, path);

	fz_var(data);
	fz_var(root);

	fz_try(ctx)
	{
		data = epub_read_file(doc, path, &len);
		root = epub_xml_root(fz_parse_xml(ctx, data, len + 1));

		/* find <body> (namespace tolerant) */
		body = NULL;
		if (root)
		{
			if (!strcmp(epub_bare_tag(root), "body"))
				body = root;
			else
			{
				for (n = fz_xml_down(root); n; n = fz_xml_next(n))
					if (!strcmp(epub_bare_tag(n), "body"))
					{ body = n; break; }
				if (!body)
					body = root; /* fallback: walk everything */
			}
		}

		if (body)
		{
			/* force a page break before each chapter */
			p.pending_break = 1;
			epub_walk(&p, body);
			epub_end_block(&p);
		}
	}
	fz_always(ctx)
	{
		fz_free_xml(ctx, root);
		fz_free(ctx, data);
		fz_free(ctx, p.dir);
	}
	fz_catch(ctx)
	{
		epub_end_block(&p);
		fz_warn(ctx, "cannot parse epub chapter: %s", path);
	}
}

/* ---- Layout: blocks -> positioned draw runs, paginated ---- */

typedef struct epub_seg_s
{
	int font;
	float size;
	float rise;
	const char *text;
	int len;
	float x; /* filled in when the line is positioned */
} epub_seg;

typedef struct epub_line_s
{
	epub_seg *seg;
	int seg_count, seg_cap;
	float width;
	float leading;
	float x; /* baseline x for the line */
	float baseline; /* baseline y */
} epub_line;

static void
epub_line_add_seg(fz_context *ctx, epub_line *ln, int font, float size, float rise,
	const char *text, int len)
{
	if (ln->seg_count == ln->seg_cap)
	{
		ln->seg_cap = ln->seg_cap ? ln->seg_cap * 2 : 8;
		ln->seg = fz_resize_array(ctx, ln->seg, ln->seg_cap, sizeof(epub_seg));
	}
	ln->seg[ln->seg_count].font = font;
	ln->seg[ln->seg_count].size = size;
	ln->seg[ln->seg_count].rise = rise;
	ln->seg[ln->seg_count].text = text;
	ln->seg[ln->seg_count].len = len;
	ln->seg[ln->seg_count].x = 0;
	ln->seg_count++;
}

static void
epub_line_reset(fz_context *ctx, epub_line *ln)
{
	(void)ctx;
	ln->seg_count = 0;
	ln->width = 0;
}

/* Append a positioned draw run to the document. */
static void
epub_emit_run(fz_context *ctx, epub_document *doc, int font, float size, float rise,
	float x, float y, const char *text, int len)
{
	if (len <= 0) return;
	if (doc->run_count == doc->run_cap)
	{
		doc->run_cap = doc->run_cap ? doc->run_cap * 2 : 256;
		doc->runs = fz_resize_array(ctx, doc->runs, doc->run_cap, sizeof(epub_draw_run));
	}
	doc->runs[doc->run_count].font = font;
	doc->runs[doc->run_count].size = size;
	doc->runs[doc->run_count].rise = rise;
	doc->runs[doc->run_count].x = x;
	doc->runs[doc->run_count].y = y;
	doc->runs[doc->run_count].text = epub_strndup(ctx, text, len);
	doc->runs[doc->run_count].len = len;
	doc->run_count++;
}

static float epub_heading_size(int level)
{
	static const float sizes[7] = { 0, 20, 16.5f, 14, 12.5f, 11.5f, 11 };
	if (level < 1 || level > 6) return EPUB_BODY_SIZE;
	return sizes[level];
}

static epub_page *epub_new_page(fz_context *ctx, epub_document *doc, int kind);

static float epub_heading_lead(int level)
{
	static const float leads[7] = { 0, 25, 21, 18, 16, 15, 14 };
	if (level < 1 || level > 6) return EPUB_BODY_LEAD;
	return leads[level];
}

/* Position the segments of a finished line and emit them as draw runs,
 advancing y and starting new pages as needed. Consumes the line. */
static void
epub_flush_line(fz_context *ctx, epub_document *doc, epub_block *b,
	epub_line *line, int *page_index, float *y, float left, float width)
{
	int i;
	float x;

	if (line->seg_count == 0)
		return;

	/* start a new page if this line does not fit */
	if (*y + line->leading > EPUB_PAGE_H - EPUB_MARGIN_B)
	{
		(*page_index)++;
		epub_new_page(ctx, doc, EPUB_PAGE_CONTENT);
		*y = EPUB_MARGIN_T;
	}

	if (b->center)
		x = left + (width - line->width) / 2;
	else if (b->right)
		x = left + (width - line->width);
	else
		x = line->x;

	line->baseline = *y + line->leading * 0.8f;

	for (i = 0; i < line->seg_count; i++)
	{
		epub_seg *seg = &line->seg[i];
		epub_emit_run(ctx, doc, seg->font, seg->size, seg->rise,
			x, line->baseline - seg->rise, seg->text, seg->len);
		x += epub_text_width(ctx, doc->fonts[seg->font], seg->text, seg->len, seg->size);
	}
	doc->pages[*page_index].run_count += line->seg_count;

	*y += line->leading;
	epub_line_reset(ctx, line);
}

/* Word-wrap one block into lines. Emits positioned runs into pages,
 handling page breaks. */
static void
epub_layout_block(fz_context *ctx, epub_document *doc, epub_block *b,
	int *page_index, float *y)
{
	epub_line line;
	float left = EPUB_MARGIN_L + b->indent;
	float width = EPUB_PAGE_W - EPUB_MARGIN_R - left;
	float avail = width;
	float marker_w = 0;
	float defsize, deflead;
	int pending_space = 0;
	epub_run *run;

	memset(&line, 0, sizeof line);

	/* image and hr blocks are handled by the caller */
	if (b->kind == EPUB_B_IMAGE || b->kind == EPUB_B_HR)
		return;

	if (b->marker)
	{
		marker_w = epub_text_width(ctx, doc->fonts[FONT_TIMES], b->marker,
			strlen(b->marker), EPUB_BODY_SIZE);
		avail -= (marker_w + 4);
	}

	if (b->kind == EPUB_B_HEADING)
	{
		defsize = epub_heading_size(b->level);
		deflead = epub_heading_lead(b->level);
	}
	else if (b->kind == EPUB_B_PRE)
	{
		defsize = EPUB_PRE_SIZE;
		deflead = EPUB_PRE_LEAD;
	}
	else
	{
		defsize = EPUB_BODY_SIZE;
		deflead = EPUB_BODY_LEAD;
	}

	/* honor an explicit page break before this block */
	if (b->page_break_before && *y > EPUB_MARGIN_T)
	{
		(*page_index)++;
		epub_new_page(ctx, doc, EPUB_PAGE_CONTENT);
		*y = EPUB_MARGIN_T;
	}

	for (run = b->runs; run; run = run->next)
	{
		int font = epub_run_font(run);
		float size = run->size > 0 ? run->size : defsize;
		int pos = 0;
		const char *s = run->text;
		int len = run->len;

		line.leading = deflead;
		line.x = left + marker_w + 4;

		if (b->kind == EPUB_B_PRE)
		{
			int start = 0, p;
			for (p = 0; p <= len; p++)
			{
				if (p == len || s[p] == '\n')
				{
					if (p > start)
						epub_line_add_seg(ctx, &line, font, size, run->rise, s + start, p - start);
					epub_flush_line(ctx, doc, b, &line, page_index, y, left, width);
					start = p + 1;
				}
			}
			continue;
		}

		/* normal word wrap: split into words on spaces */
		while (pos < len)
		{
			int ws, we;
			float wwidth, spwidth;

			while (pos < len && s[pos] == ' ') { pos++; pending_space = 1; }
			if (pos >= len) break;
			ws = pos;
			while (pos < len && s[pos] != ' ') pos++;
			we = pos;

			wwidth = epub_text_width(ctx, doc->fonts[font], s + ws, we - ws, size);
			spwidth = pending_space ? epub_text_width(ctx, doc->fonts[font], " ", 1, size) : 0;

			/* wrap if the word does not fit on the current line */
			if (line.seg_count > 0 && line.width + spwidth + wwidth > avail)
			{
				epub_flush_line(ctx, doc, b, &line, page_index, y, left, width);
				line.leading = deflead;
				line.x = left + marker_w + 4;
				pending_space = 0;
				spwidth = 0;
			}

			/* re-add the separating space (but not at line start) */
			if (pending_space && line.seg_count > 0)
			{
				epub_line_add_seg(ctx, &line, font, size, run->rise, " ", 1);
				line.width += spwidth;
			}
			pending_space = 0;

			/* if the word alone is wider than the line, hard-break it */
			if (wwidth > avail)
			{
				int p = ws;
				while (p < we)
				{
					int q = p;
					float w2 = 0;
					while (q < we)
					{
						int ucs, n = fz_chartorune(&ucs, s + q);
						float gw = epub_glyph_width(ctx, doc->fonts[font],
							fz_encode_character(ctx, doc->fonts[font], ucs), size);
						if (line.width + w2 + gw > avail && q > p) break;
						w2 += gw;
						q += n;
					}
					if (q == p) q = p + 1; /* always make progress */
					if (line.width + w2 > avail && line.seg_count > 0)
					{
						epub_flush_line(ctx, doc, b, &line, page_index, y, left, width);
						line.leading = deflead;
						line.x = left + marker_w + 4;
						w2 = 0;
					}
					epub_line_add_seg(ctx, &line, font, size, run->rise, s + p, q - p);
					line.width += w2;
					p = q;
					if (p < we && line.width >= avail - 0.01f)
					{
						epub_flush_line(ctx, doc, b, &line, page_index, y, left, width);
						line.leading = deflead;
						line.x = left + marker_w + 4;
					}
				}
				continue;
			}

			epub_line_add_seg(ctx, &line, font, size, run->rise, s + ws, we - ws);
			line.width += wwidth;
		}
		if (len > 0 && s[len-1] != ' ')
			pending_space = 0;
		else
			pending_space = 1;
	}

	/* flush the trailing partial line */
	if (line.seg_count > 0)
		epub_flush_line(ctx, doc, b, &line, page_index, y, left, width);

	(void)marker_w;

	/* trailing space after the block */
	*y += b->spacing_after;
}

/* ---- Layout driver: flow all blocks into a page array ---- */

static epub_page *
epub_new_page(fz_context *ctx, epub_document *doc, int kind)
{
	epub_page *page;
	if (doc->page_count == doc->page_cap)
	{
		doc->page_cap = doc->page_cap ? doc->page_cap * 2 : 32;
		doc->pages = fz_resize_array(ctx, doc->pages, doc->page_cap, sizeof(epub_page));
	}
	page = &doc->pages[doc->page_count++];
	page->kind = kind;
	page->first_run = doc->run_count;
	page->run_count = 0;
	page->first_image = doc->draw_image_count;
	page->image_count = 0;
	page->first_rect = doc->rect_count;
	page->rect_count = 0;
	return page;
}

static void
epub_emit_image(fz_context *ctx, epub_document *doc, fz_image *image,
	float x, float y, float w, float h)
{
	if (doc->draw_image_count == doc->draw_image_cap)
	{
		doc->draw_image_cap = doc->draw_image_cap ? doc->draw_image_cap * 2 : 64;
		doc->draw_images = fz_resize_array(ctx, doc->draw_images,
			doc->draw_image_cap, sizeof(epub_draw_image));
	}
	doc->draw_images[doc->draw_image_count].image = image;
	doc->draw_images[doc->draw_image_count].x = x;
	doc->draw_images[doc->draw_image_count].y = y;
	doc->draw_images[doc->draw_image_count].w = w;
	doc->draw_images[doc->draw_image_count].h = h;
	doc->draw_image_count++;
}

static void
epub_emit_rect(fz_context *ctx, epub_document *doc, float x0, float y0, float x1, float y1)
{
	if (doc->rect_count == doc->rect_cap)
	{
		doc->rect_cap = doc->rect_cap ? doc->rect_cap * 2 : 64;
		doc->rects = fz_resize_array(ctx, doc->rects, doc->rect_cap, sizeof(epub_draw_rect));
	}
	doc->rects[doc->rect_count].x0 = x0;
	doc->rects[doc->rect_count].y0 = y0;
	doc->rects[doc->rect_count].x1 = x1;
	doc->rects[doc->rect_count].y1 = y1;
	doc->rect_count++;
}

static void
epub_layout(fz_context *ctx, epub_document *doc)
{
	epub_block *b;
	int page_index;
	float y;

	/* title page (if we have any metadata or a cover) */
	doc->has_title_page = (doc->title || doc->author || doc->cover_path);
	if (doc->has_title_page)
		epub_new_page(ctx, doc, EPUB_PAGE_TITLE);

	page_index = doc->page_count;
	y = EPUB_MARGIN_T;
	epub_new_page(ctx, doc, EPUB_PAGE_CONTENT);

	if (doc->spine_count > 0)
	{
		int i;
		doc->chapter_start_page = fz_malloc_array(ctx, doc->spine_count, sizeof(int));
		for (i = 0; i < doc->spine_count; i++)
			doc->chapter_start_page[i] = -1;
	}

	for (b = doc->blocks; b; b = b->next)
	{
		/* record which page this chapter starts on */
		if (b->chapter >= 0 && b->chapter < doc->spine_count &&
			doc->chapter_start_page[b->chapter] == -1)
			doc->chapter_start_page[b->chapter] = page_index;

		if (b->kind == EPUB_B_IMAGE)
		{
			float iw = b->image_w, ih = b->image_h;
			float maxw = EPUB_PAGE_W - EPUB_MARGIN_L - EPUB_MARGIN_R - b->indent;
			float maxh = EPUB_CONTENT_H * 0.8f;
			float scale;
			if (iw <= 0 || ih <= 0) continue;
			scale = maxw / iw;
			if (ih * scale > maxh) scale = maxh / ih;
			if (b->page_break_before && y > EPUB_MARGIN_T)
			{
				page_index++;
				epub_new_page(ctx, doc, EPUB_PAGE_CONTENT);
				y = EPUB_MARGIN_T;
			}
			if (y + ih * scale > EPUB_PAGE_H - EPUB_MARGIN_B)
			{
				page_index++;
				epub_new_page(ctx, doc, EPUB_PAGE_CONTENT);
				y = EPUB_MARGIN_T;
			}
			epub_emit_image(ctx, doc, b->image,
				EPUB_MARGIN_L + b->indent + (maxw - iw * scale) / 2, y,
				iw * scale, ih * scale);
			doc->pages[page_index].image_count++;
			y += ih * scale + b->spacing_after;
		}
		else if (b->kind == EPUB_B_HR)
		{
			if (y + EPUB_HR_SPACE > EPUB_PAGE_H - EPUB_MARGIN_B)
			{
				page_index++;
				epub_new_page(ctx, doc, EPUB_PAGE_CONTENT);
				y = EPUB_MARGIN_T;
			}
			epub_emit_rect(ctx, doc,
				EPUB_PAGE_W / 2 - 100, y + EPUB_HR_SPACE / 2,
				EPUB_PAGE_W / 2 + 100, y + EPUB_HR_SPACE / 2 + 1);
			doc->pages[page_index].rect_count++;
			y += EPUB_HR_SPACE + b->spacing_after;
		}
		else
		{
			epub_layout_block(ctx, doc, b, &page_index, &y);
		}
	}

	/* ensure at least one page exists */
	if (doc->page_count == 0)
		epub_new_page(ctx, doc, EPUB_PAGE_CONTENT);
}

/* Resolve each toc entry's href to a page number and build the outline. */
static fz_outline *
epub_build_outline(fz_context *ctx, epub_document *doc)
{
	epub_toc_entry *e;
	fz_outline *head = NULL, *tail = NULL, *node;

	for (e = doc->toc; e; e = e->next)
	{
		int page = -1;
		int i;

		/* find the spine index for this href */
		for (i = 0; i < doc->item_count; i++)
		{
			if (!strcmp(doc->items[i].href, e->href))
			{
				int s;
				for (s = 0; s < doc->spine_count; s++)
				{
					epub_item *it = epub_find_item(doc, doc->spine[s]);
					if (it == &doc->items[i])
					{
						page = doc->chapter_start_page ? doc->chapter_start_page[s] : -1;
						break;
					}
				}
				break;
			}
		}
		if (page < 0)
			continue;

		node = fz_malloc_struct(ctx, fz_outline);
		node->title = e->title ? epub_strdup(ctx, e->title) : NULL;
		memset(&node->dest, 0, sizeof node->dest);
		node->dest.kind = FZ_LINK_GOTO;
		node->dest.ld.gotor.page = page;
		if (!head) head = node;
		else tail->next = node;
		tail = node;
	}

	return head;
}

/* ---- Rendering ---- */

static void
epub_show_run(fz_context *ctx, fz_device *dev, epub_document *doc,
	const epub_draw_run *run, const fz_matrix *ctm, float *black)
{
	fz_matrix trm;
	fz_text *text;
	fz_font *font = doc->fonts[run->font];
	const char *p = run->text;
	const char *end = run->text + run->len;
	float x = run->x;

	if (!font) return;

	trm = fz_identity;
	fz_scale(&trm, run->size, -run->size);
	text = fz_new_text(ctx, font, &trm, 0);
	fz_try(ctx)
	{
		while (p < end)
		{
			int ucs, gid;
			p += fz_chartorune(&ucs, p);
			gid = fz_encode_character(ctx, font, ucs);
			fz_add_text(ctx, text, gid, ucs, x, run->y);
			x += epub_glyph_width(ctx, font, gid, run->size);
		}
		fz_fill_text(dev, text, ctm, fz_device_rgb(ctx), black, 1);
	}
	fz_always(ctx)
	{
		fz_free_text(ctx, text);
	}
	fz_catch(ctx)
	{
		fz_rethrow(ctx);
	}
}

static void
epub_fill_rect(fz_context *ctx, fz_device *dev, const epub_draw_rect *r,
	const fz_matrix *ctm, float *color)
{
	fz_path *path = fz_new_path(ctx);
	fz_moveto(ctx, path, r->x0, r->y0);
	fz_lineto(ctx, path, r->x1, r->y0);
	fz_lineto(ctx, path, r->x1, r->y1);
	fz_lineto(ctx, path, r->x0, r->y1);
	fz_closepath(ctx, path);
	fz_fill_path(dev, path, 0, ctm, fz_device_rgb(ctx), color, 1);
	fz_free_path(ctx, path);
}

static void
epub_run_title_page(fz_context *ctx, epub_document *doc, fz_device *dev,
	const fz_matrix *ctm, float *black)
{
	float y = 160;

	if (doc->title)
	{
		epub_draw_run run;
		memset(&run, 0, sizeof run);
		run.font = FONT_TIMES_BOLD;
		run.size = 26;
		run.x = EPUB_MARGIN_L;
		run.y = y;
		run.text = doc->title;
		run.len = strlen(doc->title);
		epub_show_run(ctx, dev, doc, &run, ctm, black);
		y += 60;
	}
	if (doc->author)
	{
		epub_draw_run run;
		memset(&run, 0, sizeof run);
		run.font = FONT_TIMES_ITALIC;
		run.size = 16;
		run.x = EPUB_MARGIN_L;
		run.y = y;
		run.text = doc->author;
		run.len = strlen(doc->author);
		epub_show_run(ctx, dev, doc, &run, ctm, black);
	}
	if (doc->cover_path)
	{
		fz_image *image = epub_load_image(doc, doc->cover_path);
		if (image && image->w > 0 && image->h > 0)
		{
			float maxw = 360, maxh = 480;
			float scale = maxw / image->w;
			if (image->h * scale > maxh) scale = maxh / image->h;
			{
				fz_matrix local = *ctm;
				fz_pre_translate(&local, (EPUB_PAGE_W - image->w * scale) / 2, 280);
				fz_pre_scale(&local, image->w * scale, image->h * scale);
				fz_fill_image(dev, image, &local, 1);
			}
		}
	}
}

static void
epub_run_page(epub_document *doc, epub_page *page, fz_device *dev,
	const fz_matrix *ctm, fz_cookie *cookie)
{
	fz_context *ctx = doc->ctx;
	float black[3] = { 0, 0, 0 };
	int i, n;

	if (!page || page->number < 0 || page->number >= doc->page_count)
		return;

	if (page->kind == EPUB_PAGE_TITLE)
	{
		epub_run_title_page(ctx, doc, dev, ctm, black);
		return;
	}

	n = page->first_run + page->run_count;
	for (i = page->first_run; i < n; i++)
	{
		if (cookie && cookie->abort) return;
		epub_show_run(ctx, dev, doc, &doc->runs[i], ctm, black);
	}

	n = page->first_image + page->image_count;
	for (i = page->first_image; i < n; i++)
	{
		epub_draw_image *di = &doc->draw_images[i];
		fz_matrix local = *ctm;
		if (cookie && cookie->abort) return;
		fz_pre_translate(&local, di->x, di->y);
		fz_pre_scale(&local, di->w, di->h);
		fz_fill_image(dev, di->image, &local, 1);
	}

	n = page->first_rect + page->rect_count;
	for (i = page->first_rect; i < n; i++)
	{
		if (cookie && cookie->abort) return;
		epub_fill_rect(ctx, dev, &doc->rects[i], ctm, black);
	}
}

/* ---- Document API ---- */

static void
epub_close_document(epub_document *doc)
{
	fz_context *ctx;
	int i;
	epub_block *b, *bnext;
	epub_toc_entry *e, *enext;

	if (!doc) return;
	ctx = doc->ctx;

	for (i = 0; i < FONT_COUNT; i++)
		fz_drop_font(ctx, doc->fonts[i]);

	for (i = 0; i < doc->image_count; i++)
	{
		fz_free(ctx, doc->images[i].path);
		fz_drop_image(ctx, doc->images[i].image);
	}
	fz_free(ctx, doc->images);

	for (i = 0; i < doc->run_count; i++)
		fz_free(ctx, doc->runs[i].text);
	fz_free(ctx, doc->runs);
	fz_free(ctx, doc->draw_images);
	fz_free(ctx, doc->rects);
	fz_free(ctx, doc->pages);
	fz_free(ctx, doc->chapter_start_page);

	for (b = doc->blocks; b; b = bnext)
	{
		epub_run *r, *rn;
		bnext = b->next;
		for (r = b->runs; r; r = rn)
		{
			rn = r->next;
			fz_free(ctx, r->text);
			fz_free(ctx, r);
		}
		fz_free(ctx, b->marker);
		fz_free(ctx, b->image_path);
		fz_free(ctx, b);
	}

	for (e = doc->toc; e; e = enext)
	{
		enext = e->next;
		fz_free(ctx, e->title);
		fz_free(ctx, e->href);
		fz_free(ctx, e);
	}

	for (i = 0; i < doc->item_count; i++)
	{
		fz_free(ctx, doc->items[i].id);
		fz_free(ctx, doc->items[i].href);
		fz_free(ctx, doc->items[i].type);
		fz_free(ctx, doc->items[i].properties);
	}
	fz_free(ctx, doc->items);

	for (i = 0; i < doc->spine_count; i++)
		fz_free(ctx, doc->spine[i]);
	fz_free(ctx, doc->spine);

	for (i = 0; i < doc->entry_count; i++)
		fz_free(ctx, doc->entry[i].name);
	fz_free(ctx, doc->entry);

	fz_free(ctx, doc->opf_dir);
	fz_free(ctx, doc->title);
	fz_free(ctx, doc->author);
	fz_free(ctx, doc->cover_path);
	fz_free(ctx, doc->toc_dir);

	if (doc->file)
		fz_close(doc->file);
	fz_free(ctx, doc);
}

static int epub_count_pages(epub_document *doc) { return doc->page_count; }

static void epub_free_page(epub_document *doc, epub_page *page) { (void)doc; (void)page; }

static fz_rect *epub_bound_page(epub_document *doc, epub_page *page, fz_rect *bbox)
{
	(void)doc; (void)page;
	bbox->x0 = 0;
	bbox->y0 = 0;
	bbox->x1 = EPUB_PAGE_W;
	bbox->y1 = EPUB_PAGE_H;
	return bbox;
}

static fz_page *epub_load_page(epub_document *doc, int number)
{
	if (number < 0 || number >= doc->page_count)
		return NULL;
	doc->pages[number].number = number;
	return (fz_page *)&doc->pages[number];
}

static fz_link *epub_load_links(epub_document *doc, fz_page *page) { (void)doc; (void)page; return NULL; }
static fz_annot *epub_first_annot(epub_document *doc, fz_page *page) { (void)doc; (void)page; return NULL; }

static fz_outline *epub_load_outline(epub_document *doc)
{
	return epub_build_outline(doc->ctx, doc);
}

static int epub_meta(epub_document *doc, int key, void *ptr, int size)
{
	switch (key)
	{
	case FZ_META_FORMAT_INFO:
		fz_strlcpy((char *)ptr, "EPUB", size);
		return FZ_META_OK;
	default:
		return FZ_META_UNKNOWN_KEY;
	}
}

static void epub_rebind(epub_document *doc, fz_context *ctx)
{
	doc->ctx = ctx;
	if (doc->file)
		fz_rebind_stream(doc->file, ctx);
}

/* ---- Opening ---- */

static void
epub_load_fonts(epub_document *doc)
{
	fz_context *ctx = doc->ctx;
	static const char *names[FONT_COUNT] = {
		"Times-Roman", "Times-Bold", "Times-Italic", "Times-BoldItalic",
		"Courier"
	};
	int i;
	for (i = 0; i < FONT_COUNT; i++)
	{
		unsigned int len;
		unsigned char *data = pdf_lookup_builtin_font(names[i], &len);
		if (data)
			doc->fonts[i] = fz_new_font_from_memory(ctx, names[i], data, len, 0, 1);
	}
}

/* Locate META-INF/container.xml and return the OPF path (fz_malloc'd). */
static char *
epub_find_opf(epub_document *doc)
{
	fz_context *ctx = doc->ctx;
	unsigned char *data;
	fz_xml *root, *node, *rootfile;
	char *fullpath = NULL;
	int len;

	fz_var(data);

	fz_try(ctx)
	{
		data = epub_read_file(doc, "META-INF/container.xml", &len);
		root = epub_xml_root(fz_parse_xml(ctx, data, len + 1));
		if (root)
		{
			node = epub_find_child(root, "rootfiles");
			if (node)
			{
				rootfile = epub_find_child(node, "rootfile");
				if (rootfile)
				{
					char *path = fz_xml_att(rootfile, "full-path");
					if (path)
						fullpath = epub_strdup(ctx, path);
				}
			}
			fz_free_xml(ctx, root);
		}
		fz_free(ctx, data);
	}
	fz_catch(ctx)
	{
		fullpath = NULL;
	}

	/* fall back: look for any .opf entry */
	if (!fullpath)
	{
		int i;
		for (i = 0; i < doc->entry_count; i++)
		{
			char *ext = strrchr(doc->entry[i].name, '.');
			if (ext && !fz_strcasecmp(ext, ".opf"))
			{
				fullpath = epub_strdup(ctx, doc->entry[i].name);
				break;
			}
		}
	}

	return fullpath;
}

static void
epub_init_document(epub_document *doc)
{
	doc->super.close = (fz_document_close_fn *)epub_close_document;
	doc->super.count_pages = (fz_document_count_pages_fn *)epub_count_pages;
	doc->super.load_page = (fz_document_load_page_fn *)epub_load_page;
	doc->super.bound_page = (fz_document_bound_page_fn *)epub_bound_page;
	doc->super.run_page_contents = (fz_document_run_page_contents_fn *)epub_run_page;
	doc->super.free_page = (fz_document_free_page_fn *)epub_free_page;
	doc->super.load_links = (fz_document_load_links_fn *)epub_load_links;
	doc->super.first_annot = (fz_document_first_annot_fn *)epub_first_annot;
	doc->super.load_outline = (fz_document_load_outline_fn *)epub_load_outline;
	doc->super.meta = (fz_document_meta_fn *)epub_meta;
	doc->super.rebind = (fz_document_rebind_fn *)epub_rebind;
}

static epub_document *
epub_open_document_with_stream(fz_context *ctx, fz_stream *file)
{
	epub_document *doc;
	char *opf_path = NULL;
	unsigned char *opf_data = NULL;
	int i, opf_len;

	doc = fz_malloc_struct(ctx, epub_document);
	doc->ctx = ctx;
	doc->file = file;
	fz_var(opf_path);
	fz_var(opf_data);

	fz_try(ctx)
	{
		epub_init_document(doc);
		epub_read_zip_dir(doc);
		epub_load_fonts(doc);

		opf_path = epub_find_opf(doc);
		if (!opf_path)
			fz_throw(ctx, FZ_ERROR_GENERIC, "cannot find epub package document");

		doc->opf_dir = epub_dirname(ctx, opf_path);

		opf_data = epub_read_file(doc, opf_path, &opf_len);
		epub_parse_opf(doc, opf_data, opf_len);
		fz_free(ctx, opf_data);
		opf_data = NULL;

		/* fallback: if no spine entries were found, treat every
		   xhtml/html manifest item as a chapter (document order) */
		if (doc->spine_count == 0)
		{
			int cap = 16, i;
			doc->spine = fz_malloc_array(ctx, cap, sizeof(char *));
			for (i = 0; i < doc->item_count; i++)
			{
				char *type = doc->items[i].type;
				char *href = doc->items[i].href;
				if (type && (!strcmp(type, "application/xhtml+xml") ||
						!strcmp(type, "text/html")) &&
					href && href[0])
				{
					if (doc->spine_count == cap)
					{
						cap *= 2;
						doc->spine = fz_resize_array(ctx, doc->spine, cap, sizeof(char *));
					}
					doc->spine[doc->spine_count++] = epub_strdup(ctx, doc->items[i].id);
				}
			}
		}

		/* parse all spine chapters into blocks */
		for (i = 0; i < doc->spine_count; i++)
		{
			epub_item *item = epub_find_item(doc, doc->spine[i]);
			if (item && item->href)
				epub_parse_chapter(doc, i, item->href);
		}

		/* paginate */
		epub_layout(ctx, doc);
	}
	fz_catch(ctx)
	{
		fz_free(ctx, opf_path);
		fz_free(ctx, opf_data);
		doc->file = NULL; /* caller owns the stream on failure */
		epub_close_document(doc);
		fz_rethrow(ctx);
	}

	fz_free(ctx, opf_path);
	return doc;
}

static epub_document *
epub_open_document(fz_context *ctx, const char *filename)
{
	fz_stream *stm = fz_open_file(ctx, filename);
	epub_document *doc;
	fz_try(ctx)
	{
		doc = epub_open_document_with_stream(ctx, stm);
	}
	fz_catch(ctx)
	{
		fz_close(stm);
		fz_rethrow(ctx);
	}
	return doc;
}

static int
epub_recognize(fz_context *ctx, const char *magic)
{
	char *ext = strrchr(magic, '.');
	(void)ctx;
	if (ext && (!fz_strcasecmp(ext, ".epub") || !fz_strcasecmp(ext, ".epub3")))
		return 100;
	if (!strcmp(magic, "epub") || !strcmp(magic, "application/epub+zip"))
		return 100;
	return 0;
}

fz_document_handler epub_document_handler =
{
	(fz_document_recognize_fn *)&epub_recognize,
	(fz_document_open_fn *)&epub_open_document,
	(fz_document_open_with_stream_fn *)&epub_open_document_with_stream
};
