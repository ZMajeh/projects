#ifndef MUPDF_FITZ_OUTPUT_SVG_H
#define MUPDF_FITZ_OUTPUT_SVG_H

#include "majehpdfviewer/fitz/system.h"
#include "majehpdfviewer/fitz/context.h"
#include "majehpdfviewer/fitz/device.h"
#include "majehpdfviewer/fitz/output.h"

fz_device *fz_new_svg_device(fz_context *ctx, fz_output *out, float page_width, float page_height);

#endif
