#!/bin/bash
# Majeh's PDF Viewer - Windows Cross-Build Script (run from Linux/WSL)
# Requires: mingw-w64  (sudo apt install mingw-w64)
set -e
echo "Cross-building Majeh's PDF Viewer for Windows (x86_64)..."

echo "--- Compiling libraries and tools..."
make OS=w64_amd64-cross-mingw32 build=release -j"$(nproc)"

echo "--- Building Windows resource object..."
x86_64-w64-mingw32-windres platform/x11/win_res.rc -O coff -o build/release/win_res.obj

echo "--- Linking GUI (release)..."
x86_64-w64-mingw32-gcc -Iinclude -Igenerated -Wall -pipe -O1 -DNDEBUG \
	-o build/release/majehs-viewer.exe \
	platform/x11/win_main.c platform/x11/pdfapp.c build/release/win_res.obj \
	build/release/libmajehpdfviewer.a \
	build/release/libfreetype.a build/release/libjbig2dec.a build/release/libjpeg.a \
	build/release/libopenjpeg.a build/release/libz.a build/release/libmujs.a \
	-lgdi32 -lcomdlg32 -lcomctl32 -lwinspool -lm -mwindows

echo "--- Stripping symbols for smaller binary..."
x86_64-w64-mingw32-strip build/release/majehs-viewer.exe

echo "Build Complete: build/release/majehs-viewer.exe"
