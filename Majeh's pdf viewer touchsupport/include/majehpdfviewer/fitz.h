#ifndef MUDPF_FITZ_H
#define MUDPF_FITZ_H

#include "majehpdfviewer/fitz/version.h"
#include "majehpdfviewer/fitz/system.h"
#include "majehpdfviewer/fitz/context.h"

#include "majehpdfviewer/fitz/crypt.h"
#include "majehpdfviewer/fitz/getopt.h"
#include "majehpdfviewer/fitz/hash.h"
#include "majehpdfviewer/fitz/math.h"
#include "majehpdfviewer/fitz/string.h"
#include "majehpdfviewer/fitz/tree.h"
#include "majehpdfviewer/fitz/xml.h"

/* I/O */
#include "majehpdfviewer/fitz/buffer.h"
#include "majehpdfviewer/fitz/stream.h"
#include "majehpdfviewer/fitz/compressed-buffer.h"
#include "majehpdfviewer/fitz/filter.h"
#include "majehpdfviewer/fitz/output.h"

/* Resources */
#include "majehpdfviewer/fitz/store.h"
#include "majehpdfviewer/fitz/colorspace.h"
#include "majehpdfviewer/fitz/pixmap.h"
#include "majehpdfviewer/fitz/glyph.h"
#include "majehpdfviewer/fitz/bitmap.h"
#include "majehpdfviewer/fitz/image.h"
#include "majehpdfviewer/fitz/function.h"
#include "majehpdfviewer/fitz/shade.h"
#include "majehpdfviewer/fitz/font.h"
#include "majehpdfviewer/fitz/path.h"
#include "majehpdfviewer/fitz/text.h"

#include "majehpdfviewer/fitz/device.h"
#include "majehpdfviewer/fitz/display-list.h"
#include "majehpdfviewer/fitz/structured-text.h"

#include "majehpdfviewer/fitz/transition.h"
#include "majehpdfviewer/fitz/glyph-cache.h"

/* Document */
#include "majehpdfviewer/fitz/link.h"
#include "majehpdfviewer/fitz/outline.h"
#include "majehpdfviewer/fitz/document.h"
#include "majehpdfviewer/fitz/annotation.h"
#include "majehpdfviewer/fitz/meta.h"

#include "majehpdfviewer/fitz/write-document.h"

/* Output formats */
#include "majehpdfviewer/fitz/output-pnm.h"
#include "majehpdfviewer/fitz/output-png.h"
#include "majehpdfviewer/fitz/output-pwg.h"
#include "majehpdfviewer/fitz/output-pcl.h"
#include "majehpdfviewer/fitz/output-svg.h"
#include "majehpdfviewer/fitz/output-tga.h"

#endif
