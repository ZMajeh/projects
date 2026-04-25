#!/bin/bash
# Majeh's PDF Viewer - Linux Build Script
echo "Building Majeh's PDF Viewer..."
make -j$(nproc)
echo "Build Complete: build/debug/mupdf-x11"
