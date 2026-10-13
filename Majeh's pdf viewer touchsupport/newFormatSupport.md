# Adding a New Document Format

How to teach this viewer a new file type — using the EPUB handler as the
reference example. Everything discussed here lives in the `source/` and
`include/` directories (the `platform/x11` GUI bits change only where a file
dialog filter is needed).

---

## 1. How it works (the short version)

The document layer is a **plugin list**. `fz_document_handler` is a small
struct with three function pointers:

```c
struct fz_document_handler_s
{
    fz_document_recognize_fn *recognize;  // returns 1/100 if it handles the file
    fz_document_open_fn *open;            // open by filename
    fz_document_open_with_stream_fn *open_with_stream; // open from a stream
};
```

At startup, `fz_register_document_handlers()` registers every handler in the
list. When you drop (or double-click) a file, MuPDF asks each handler, in order,
"do you know this file?" — comparing the filename's extension (and sometimes a
magic byte check) — and uses the one with the highest score. The rest of the
engine (page counting, page loading, drawing, outlines, metadata) is handled by
calling the functions your handler fills in.

That means adding a format is purely **additive**: a new file, a new
`fz_document_handler`, a registration, and a Makefile entry. You do **not**
touch the existing PDF/XPS/CBZ/… code.

---

## 2. Files you must touch

The table below uses EPUB as the concrete example.

| File | What changes | Why |
|------|--------------|-----|
| `include/majehpdfviewer/epub.h` *(new)* | Declare the handler's API: `epub_document`, `epub_page`, and `epub_document *epub_open_document(fz_context *ctx, const char *filename)`, `epub_open_document_with_stream`, `epub_close_document`, `epub_count_pages`, `epub_load_page`, `epub_bound_page`, `epub_free_page`, `epub_run_page`, `epub_load_outline`, `epub_meta`. | The library users (mutool, mudraw) and the app call these functions. |
| `source/epub/muepub.c` *(new)* | The implementation of the API above. | The entire document logic for this format. |
| `include/majehpdfviewer/fitz/document.h` *(edit)* | Add `extern fz_document_handler epub_document_handler;` next to the other `extern` declarations. | So the app can reference the handler. |
| `source/fitz/document-all.c` *(edit)* | Add `fz_register_document_handler(ctx, &epub_document_handler);` inside `fz_register_document_handlers()`. | Tells the engine the handler exists. |
| `Makefile` *(edit)* | Add `source/epub/` to `ALL_DIR`, `EPUB_SRC`/`EPUB_OBJ`/`EPUB_HDR` lists, a `-include` rule for the header, and add `$(EPUB_OBJ)` to `MUPDF_LIB`. | So the library links it. |
| `build.ps1` *(edit, Windows only)* | Add `"epub"` to the `$dirs` array. | So the Windows build compiles it. |
| `platform/x11/win_main.c` *(edit, optional)* | Add `*.epub` to the Open/Save dialog file filters. | The file picker shows EPUB files. |

That is it. Everything else (the MuPDF engine, the drawing device, the page
model) already supports whatever your handler hands it — a `fz_page` that
returns a `fz_rect` (the mediabox), a `run_page_contents` that draws text and
images, and (optionally) an outline and metadata.
## 3. Step-by-step

### 3.1. Header + implementation skeleton

Create `include/majehpdfviewer/epub.h` with the same shape as `cbz.h`:

```c
#ifndef MAJEH_EPUB_H
#define MAJEH_EPUB_H

#include "majehpdfviewer/fitz.h"

typedef struct epub_document_s epub_document;
typedef struct epub_page_s epub_page;

epub_document *epub_open_document(fz_context *ctx, const char *filename);
epub_document *epub_open_document_with_stream(fz_context *ctx, fz_stream *file);
void epub_close_document(epub_document *doc);
int epub_count_pages(epub_document *doc);
epub_page *epub_load_page(epub_document *doc, int number);
fz_rect *epub_bound_page(epub_document *doc, epub_page *page, fz_rect *rect);
void epub_free_page(epub_document *doc, epub_page *page);
void epub_run_page(epub_document *doc, epub_page *page, fz_device *dev,
                   const fz_matrix *ctm, fz_cookie *cookie);

#endif
```

### 3.2. Document structure

Keep the document struct self-contained (no globals):

```c
struct epub_document_s
{
    fz_document super;
    fz_context *ctx;
    fz_stream *file;               /* stream, owned by the document */
    /* format-specific state: zip entries, manifest, spine, stylesheets, fonts */
};
```

`open_with_stream` should open the stream once and store it in `doc->file`.
`close` must close the stream and free everything, and must be safe to call on
a partially-initialized document (set `doc->file = NULL` before `fz_close`).

### 3.3. The `recognize` callback

Match by **extension first**, then by content. A real implementation looks like:

```c
static int
epub_recognize(fz_context *ctx, const char *magic)
{
    char *ext = strrchr(magic, '.');
    (void)ctx;
    if (ext && (!fz_strcasecmp(ext, ".epub"))) return 100;
    if (!strcmp(magic, "epub") || !strcmp(magic, "application/epub+zip")) return 100;
    return 0;
}
```

Return **100** for a very specific match, lower for ambiguous, and **0** to say
"not this format". The `open` function is only called when `recognize` returned
nonzero.

### 3.4. Open + parse

Inside `epub_open_document_with_stream`:

1. `doc = fz_malloc_struct(ctx, epub_document)`.
2. Store `doc->ctx = ctx`, `doc->file = file`.
3. Read the **package metadata** (OPF, MIME types, outline, cover).
4. Fill `doc->super` function pointers:
   ```c
   doc->super.close = (fz_document_close_fn *)epub_close_document;
   doc->super.count_pages = (fz_document_count_pages_fn *)epub_count_pages;
   doc->super.load_page = (fz_document_load_page_fn *)epub_load_page;
   doc->super.bound_page = (fz_document_bound_page_fn *)epub_bound_page;
   doc->super.run_page_contents = (fz_document_run_page_contents_fn *)epub_run_page;
   doc->super.load_outline = (fz_document_load_outline_fn *)epub_load_outline;
   doc->super.meta = (fz_document_meta_fn *)epub_meta;
   doc->super.rebind = (fz_document_rebind_fn *)epub_rebind;
   ```
5. On any failure: `fz_throw(ctx, FZ_ERROR_GENERIC, "reason")`; the caller
   catches it. Never leak memory.

### 3.5. `recognize` + registration

Add the handler struct at file scope:

```c
fz_document_handler epub_document_handler =
{
    (fz_document_recognize_fn *)&epub_recognize,
    (fz_document_open_fn *)&epub_open_document,
    (fz_document_open_with_stream_fn *)&epub_open_document_with_stream
};
```

Register it in `source/fitz/document-all.c`:

```c
void fz_register_document_handlers(fz_context *ctx)
{
    fz_register_document_handler(ctx, &pdf_document_handler);
    fz_register_document_handler(ctx, &xps_document_handler);
    fz_register_document_handler(ctx, &cbz_document_handler);
    fz_register_document_handler(ctx, &img_document_handler);
    fz_register_document_handler(ctx, &tiff_document_handler);
    fz_register_document_handler(ctx, &txt_document_handler);
    fz_register_document_handler(ctx, &epub_document_handler);  /* <-- add */
}
```

### 3.6. Build wiring

* `Makefile`: `ALL_DIR += $(OUT)/epub`, add `source/epub/*.c` to `EPUB_SRC`,
  add a header-include rule, add `$(EPUB_OBJ)` to `MUPDF_LIB`.
* `build.ps1` (Windows): add `"epub"` to the `$dirs` array. The source/epub
  folder will then be compiled for Windows too.

### 3.7. File dialogs (Windows)

In `platform/x11/win_main.c` add `*.epub` to the Open/Save filter strings so
the user can pick one. Optional but expected.




<!-- CONTINUE-README -->

---

## 4. Quick checklist

- [ ] `epub.h` declares the document API
- [ ] `muepub.c` implements `open/_with_stream/_close`, `count_pages`,
      `load_page`, `bound_page`, `run_page`, `load_outline`, `meta`
- [ ] `recognize` returns 100 for the file extension (+ magic-byte fallback)
- [ ] `fz_document_handler epub_document_handler = …` declared at file scope
- [ ] registered in `document-all.c`
- [ ] `Makefile` compiles `source/epub/*.c` into the library
- [ ] `build.ps1` (Windows) compiles it too
- [ ] file dialogs show `*.ext`
- [ ] render at least one page to PNG and confirm text + images appear

---

## 5. Dos and Don'ts

### Do

* **Keep the document struct self-contained.** Everything the format needs
  (entries, manifest, spine, fonts, stylesheets, outlines) lives in the struct;
  never use a global.
* **Close everything in `close`.** Streams, buffers, fonts, images, XML trees,
  blocks, runs, outlines, TOC entries, items, spine, and entry names. The app
  calls `fz_close` on the stream it opened itself if `open_with_stream` was
  used, but ownership is yours here.
* **Set `doc->super` only when the document opened successfully.** If parsing
  throws (or fails), call `fz_throw(ctx, FZ_ERROR_GENERIC, "reason")` and let
  the caller fall back.
* **Return a simple mediabox for every page** — `612 x 792` (letter) is fine if
  the format has no notion of pages.
* **Cache expensive resources** (decoded images, fonts, parsed XML, TOC entries)
  in the document struct and free them all in `close`.
* **Keep text as UTF-8.** The engine passes UTF-8 runs to the device. Convert
  your format's internal encoding at parse time.
* **Parse in one pass when you can.** The CBZ/EPUB handler reads the zip directory
  first, then reads entries on demand. Libraries like `libxml2` are fine but
  make sure you free the tree in `close`.

### Don't

* **Don't assume your format is "read-only-friendly".** MuPDF may open the file
  read-only, stream it, or rebind the context (`fz_rebind_stream`); keep
  `doc->file` a stream and rebind it in `rebind`.
* **Don't leak.** Every `fz_malloc`/`fz_malloc_array`/`fz_strdup` returned to
  the engine must have a matching `fz_free` in `close`. The project has no
  garbage collector.
* **Don't block the document thread for a long time in `open`.** If parsing is
  expensive, do it lazily on first `load_page` and cache the result.
* **Don't call `fz_open_document` recursively.** Handlers are for *opening* a
  file; if you need a second file (a stylesheet, a child document), open it
  through its own `fz_*` function instead of recursing into `fz_open_document`.
* **Don't hard-code encodings or break on real-world files.** EPUBs use
  `xml:lang`, `xmlns:*`, `dc:*` prefixes, `%XX` in paths, inline `style="…"`,
  and namespace-aware tags (`<opf:package>`). Match by *title* (strip the
  namespace), not by exact tag string.
* **Don't skip `load_outline` unless it's meaningless.** The outline is what
  lets the user jump around and adds an "outline" presentation to the TOC view.
  A plain text file whose "outline" is just paragraph numbers is still useful.
* **Don't assume `count_pages` is cheap.** It runs at open time in many places;
  keep it O(1) or O(pages) with a small constant.
* **Don't put the format's logic in `source/fitz/` (the engine layer).** The
  engine layer is shared and version-pinned. Format code belongs in a new
  `source/<format>/` directory, exactly as `cbz/` and `muepub.c` do.
* **Don't rely on `magic` being a filename.** It is the value the recognizer is
  checking (often a filename, but can be a magic string or stream hint).
  Handle `NULL` gracefully.

---

## 6. Why "table of contents" and "metadata" matter

The MuPDF engine presents pages and objects through `fz_document`. Two of its
properties are optional but usually worth filling in, because they drive the
viewer's UI:

* `meta(FZ_META_FORMAT_INFO, …)` → `"EPUB"`, `"PDF"`, … shown in the status bar
  and used by tools (`mutool info`).
* `load_outline` → the document's hierarchical TOC (chapters, sections). The
  viewer renders a clickable outline for PDFs; an EPUB with just a `-1/page-0`
  outline is effectively broken from the user's point of view. Always fill it
