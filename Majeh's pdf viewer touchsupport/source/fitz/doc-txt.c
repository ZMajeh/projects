#include "majehpdfviewer/fitz.h"
#include "majehpdfviewer/pdf.h"

#include <string.h>
#include <stdio.h>

typedef struct txt_document_s txt_document;
typedef struct txt_page_s txt_page;

struct txt_document_s
{
	fz_document super;
	fz_context *ctx;
	fz_buffer *buf;
	int *page_offsets;
	int page_count;
};

struct txt_page_s
{
	txt_document *txtdoc;
	int number;
};

typedef struct {
	char *buf_start;
	char *s;
	char *end;
	char *line_start;
	char *line_end;
	int is_new_logical;
} txt_line_iterator;

static int txt_next_display_line(fz_context *ctx, fz_font *font, float size, float max_w, txt_line_iterator *it)
{
	if (!it->s || it->s >= it->end) return 0;
	
	it->line_start = it->s;
	it->is_new_logical = (it->s == it->buf_start || it->s[-1] == '\n' || it->s[-1] == '\r');
	
	float cur_w = 0;
	char *last_space = NULL;
	char *p = it->s;
	
	while (p < it->end && *p != '\n' && *p != '\r')
	{
		int ucs, gid;
		int n = fz_chartorune(&ucs, p);
		if (ucs == '\t') {
			ucs = ' ';
			gid = fz_encode_character(ctx, font, ucs);
			float w = fz_advance_glyph(ctx, font, gid) * size;
			if (w > 100) w /= 1000.0f;
			w *= 4; // 4-space tabs
			if (cur_w + w > max_w) break;
			cur_w += w;
			p += n;
			continue;
		}
		if (ucs < 32 && ucs != '\n' && ucs != '\r' && ucs != '\t') ucs = '.';
		gid = fz_encode_character(ctx, font, ucs);
		float w = fz_advance_glyph(ctx, font, gid) * size;
		if (w > 100) w /= 1000.0f; // Handle EM units vs pre-scaled
		
		if (cur_w + w > max_w)
		{
			if (last_space && last_space > it->line_start)
			{
				it->line_end = last_space + 1;
				it->s = last_space + 1;
			}
			else
			{
				if (p == it->line_start) p += n;
				it->line_end = p;
				it->s = p;
			}
			return 1;
		}
		
		if (*p == ' ') last_space = p;
		cur_w += w;
		p += n;
	}
	
	it->line_end = p;
	it->s = p;
	if (it->s < it->end && *it->s == '\r') {
		it->s++;
		if (it->s < it->end && *it->s == '\n') it->s++;
	} else if (it->s < it->end && *it->s == '\n') {
		it->s++;
	}
	return 1;
}

static void
txt_close_document(txt_document *doc)
{
	fz_warn(doc->ctx, "txt_close_document");
	fz_drop_buffer(doc->ctx, doc->buf);
	fz_free(doc->ctx, doc->page_offsets);
	fz_free(doc->ctx, doc);
}

static int
txt_count_pages(txt_document *doc)
{
	return doc->page_count;
}

static fz_rect *
txt_bound_page(txt_document *doc, txt_page *page, fz_rect *bbox)
{
	bbox->x0 = 0;
	bbox->y0 = 0;
	bbox->x1 = 612;
	bbox->y1 = 792;
	return bbox;
}

static void
txt_show_string(fz_context *ctx, fz_text *text, char *s, int len, float x, float y, float size)
{
	int gid, ucs;
	char *p = s;
	char *end = s + len;
	while (p < end)
	{
		p += fz_chartorune(&ucs, p);
		if (ucs == '\t') {
			ucs = ' ';
			gid = fz_encode_character(ctx, text->font, ucs);
			float w = fz_advance_glyph(ctx, text->font, gid) * size;
			if (w > 100) w /= 1000.0f;
			for (int i=0; i<4; i++) {
				fz_add_text(ctx, text, gid, ucs, x, y);
				x += w;
			}
			continue;
		}
		if (ucs < 32 && ucs != '\n' && ucs != '\r' && ucs != '\t') ucs = '.';
		gid = fz_encode_character(ctx, text->font, ucs);
		fz_add_text(ctx, text, gid, ucs, x, y);
		float w = fz_advance_glyph(ctx, text->font, gid) * size;
		if (w > 100) w /= 1000.0f;
		x += w;
	}
}

static float
txt_string_width(fz_context *ctx, fz_font *font, char *s, int len, float size)
{
	float w = 0;
	int ucs, gid;
	char *p = s;
	char *end = s + len;
	while (p < end)
	{
		p += fz_chartorune(&ucs, p);
		if (ucs == '\t') {
			ucs = ' ';
			gid = fz_encode_character(ctx, font, ucs);
			float aw = fz_advance_glyph(ctx, font, gid) * size;
			if (aw > 100) aw /= 1000.0f;
			w += aw * 4;
			continue;
		}
		if (ucs < 32 && ucs != '\n' && ucs != '\r' && ucs != '\t') ucs = '.';
		gid = fz_encode_character(ctx, font, ucs);
		float aw = fz_advance_glyph(ctx, font, gid) * size;
		if (aw > 100) aw /= 1000.0f;
		w += aw;
	}
	return w;
}

static void
txt_fill_rect(fz_context *ctx, fz_device *dev, const fz_rect *rect, const fz_matrix *ctm, fz_colorspace *cs, float *color, float alpha)
{
	fz_path *path = fz_new_path(ctx);
	fz_moveto(ctx, path, rect->x0, rect->y0);
	fz_lineto(ctx, path, rect->x1, rect->y0);
	fz_lineto(ctx, path, rect->x1, rect->y1);
	fz_lineto(ctx, path, rect->x0, rect->y1);
	fz_closepath(ctx, path);
	fz_fill_path(dev, path, 0, ctm, cs, color, alpha);
	fz_free_path(ctx, path);
}

static void
txt_run_page(txt_document *doc, txt_page *page, fz_device *dev, const fz_matrix *ctm, fz_cookie *cookie)
{
	fz_context *ctx = doc->ctx;
	fz_font *font;
	fz_text *text, *num_text;
	fz_matrix trm;
	float x = 40, y = 50;
	int display_line_count = 0;
	unsigned int font_len;
	unsigned char *font_data;
	float black[3] = { 0, 0, 0 };
	fz_rect ruler_rect = { 34, 20, 35, 772 };
	txt_line_iterator it;

	if (!doc->buf) return;
	if (!page || page->number < 0 || page->number >= doc->page_count) return;

	font_data = pdf_lookup_builtin_font("Courier", &font_len);
	if (font_data)
		font = fz_new_font_from_memory(ctx, "Courier", font_data, font_len, 0, 0);
	else
		font = fz_load_system_font(ctx, "Courier", 0, 0, 0);

	if (!font) return;

	txt_fill_rect(ctx, dev, &ruler_rect, ctm, fz_device_rgb(ctx), black, 1);

	trm = fz_identity;
	fz_scale(&trm, 12, -12);
	text = fz_new_text(ctx, font, &trm, 0);

	trm = fz_identity;
	fz_scale(&trm, 8, -8);
	num_text = fz_new_text(ctx, font, &trm, 0);

	it.buf_start = (char *)doc->buf->data;
	it.s = (char *)doc->buf->data + doc->page_offsets[page->number];
	it.end = (char *)doc->buf->data + doc->buf->len;

	fz_try(ctx)
	{
		int logical_line_number = 1;
		char *p = it.buf_start;
		while (p < it.s) {
			if (*p == '\r') {
				logical_line_number++;
				if (p + 1 < it.s && p[1] == '\n') p++;
			} else if (*p == '\n') {
				logical_line_number++;
			}
			p++;
		}

		while (txt_next_display_line(ctx, font, 12, 612 - 40 - 40, &it) && display_line_count < 60)
		{
			if (it.is_new_logical) {
				char num_buf[16];
				sprintf(num_buf, "%d", logical_line_number++);
				float nx = 30 - txt_string_width(ctx, font, num_buf, strlen(num_buf), 8);
				txt_show_string(ctx, num_text, num_buf, strlen(num_buf), nx, y, 8);
			}

			txt_show_string(ctx, text, it.line_start, it.line_end - it.line_start, x, y, 12);
			y += 12;
			display_line_count++;
		}
		fz_fill_text(dev, num_text, ctm, fz_device_rgb(ctx), black, 1);
		fz_fill_text(dev, text, ctm, fz_device_rgb(ctx), black, 1);
	}
	fz_always(ctx)
	{
		fz_free_text(ctx, num_text);
		fz_free_text(ctx, text);
		fz_drop_font(ctx, font);
	}
	fz_catch(ctx)
	{
		fz_rethrow(ctx);
	}
}

static void
txt_layout(txt_document *doc, fz_font *font)
{
	fz_context *ctx = doc->ctx;
	int display_line_count = 0;
	int page_cap = 10;
	txt_line_iterator it;
	
	doc->page_offsets = fz_malloc_array(ctx, page_cap, sizeof(int));
	doc->page_offsets[0] = 0;
	doc->page_count = 1;

	if (!doc->buf) return;

	it.buf_start = (char *)doc->buf->data;
	it.s = (char *)doc->buf->data;
	it.end = (char *)doc->buf->data + doc->buf->len;

	while (txt_next_display_line(ctx, font, 12, 612 - 40 - 40, &it))
	{
		if (display_line_count == 60)
		{
			if (doc->page_count == page_cap)
			{
				page_cap *= 2;
				doc->page_offsets = fz_resize_array(ctx, doc->page_offsets, page_cap, sizeof(int));
			}
			doc->page_offsets[doc->page_count++] = it.line_start - it.buf_start;
			display_line_count = 0;
		}
		display_line_count++;
	}
}

static fz_page *
txt_load_page(txt_document *doc, int number)
{
	txt_page *page = fz_malloc_struct(doc->ctx, txt_page);
	page->txtdoc = doc;
	page->number = number;
	return (fz_page *)page;
}

static void
txt_free_page(txt_document *doc, txt_page *page)
{
	fz_free(doc->ctx, page);
}

static fz_link *
txt_load_links(txt_document *doc, fz_page *page)
{
	return NULL;
}

static fz_annot *
txt_first_annot(txt_document *doc, fz_page *page)
{
	return NULL;
}

static int
txt_meta(txt_document *doc, int key, void *ptr, int size)
{
	switch(key)
	{
	case FZ_META_FORMAT_INFO:
		fz_strlcpy((char *)ptr, "TEXT", size);
		return 0;
	default:
		return -1;
	}
}

static fz_document *
txt_open_document_with_stream(fz_context *ctx, fz_stream *stm)
{
	txt_document *doc = fz_malloc_struct(ctx, txt_document);
	unsigned int font_len;
	unsigned char *font_data;
	fz_font *font;

	doc->super.close = (fz_document_close_fn *)txt_close_document;
	doc->super.count_pages = (fz_document_count_pages_fn *)txt_count_pages;
	doc->super.load_page = (fz_document_load_page_fn *)txt_load_page;
	doc->super.bound_page = (fz_document_bound_page_fn *)txt_bound_page;
	doc->super.run_page_contents = (fz_document_run_page_contents_fn *)txt_run_page;
	doc->super.free_page = (fz_document_free_page_fn *)txt_free_page;
	doc->super.load_links = (fz_document_load_links_fn *)txt_load_links;
	doc->super.first_annot = (fz_document_first_annot_fn *)txt_first_annot;
	doc->super.meta = (fz_document_meta_fn *)txt_meta;
	doc->ctx = ctx;
	doc->page_offsets = NULL;
	doc->page_count = 0;

	fz_try(ctx)
	{
		if (stm)
			doc->buf = fz_read_all(stm, 1024);
		else
			doc->buf = fz_new_buffer(ctx, 1);
		
		font_data = pdf_lookup_builtin_font("Courier", &font_len);
		if (font_data)
			font = fz_new_font_from_memory(ctx, "Courier", font_data, font_len, 0, 0);
		else
			font = fz_load_system_font(ctx, "Courier", 0, 0, 0);
		
		if (font) {
			txt_layout(doc, font);
			fz_drop_font(ctx, font);
		} else {
			doc->page_offsets = fz_malloc_array(ctx, 1, sizeof(int));
			doc->page_offsets[0] = 0;
			doc->page_count = 1;
		}
	}
	fz_catch(ctx)
	{
		if (doc->page_offsets) fz_free(ctx, doc->page_offsets);
		if (doc->buf) fz_drop_buffer(ctx, doc->buf);
		fz_free(ctx, doc);
		fz_rethrow(ctx);
	}
	return (fz_document *)doc;
}

static fz_document *
txt_open_document(fz_context *ctx, const char *filename)
{
	fz_stream *stm = fz_open_file(ctx, filename);
	fz_document *doc;
	fz_try(ctx)
	{
		doc = txt_open_document_with_stream(ctx, stm);
	}
	fz_always(ctx)
	{
		fz_close(stm);
	}
	fz_catch(ctx)
	{
		fz_rethrow(ctx);
	}
	return doc;
}

static int
txt_recognize(fz_context *ctx, const char *magic)
{
	char *ext = strrchr(magic, '.');
	if (ext && (!fz_strcasecmp(ext, ".txt") || !fz_strcasecmp(ext, ".reg") || !fz_strcasecmp(ext, ".ini") || !fz_strcasecmp(ext, ".log") || !fz_strcasecmp(ext, ".inf") || !fz_strcasecmp(ext, ".bat") || !fz_strcasecmp(ext, ".cmd")))
		return 100;
	return 1;
}

fz_document_handler txt_document_handler =
{
	txt_recognize,
	txt_open_document,
	txt_open_document_with_stream
};
