#!/bin/bash
# Majeh's PDF Viewer - Linux Build Script
set -e
echo "Building Majeh's PDF Viewer..."
make -j"$(nproc)"
echo "Build Complete: build/debug/majehs-viewer"
