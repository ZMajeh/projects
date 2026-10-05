#!/bin/bash
# Majeh's PDF Viewer - Linux Dependency Installer

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (use sudo)"
  exit
fi

if [ -f /etc/debian_version ]; then
    echo "Detected Debian/Ubuntu..."
    apt-get update
    apt-get install -y build-essential pkg-config libx11-dev libxext-dev libcurl4-openssl-dev zlib1g-dev libfreetype6-dev libjpeg-dev libopenjpeg-dev libjbig2dec0-dev
elif [ -f /etc/fedora-release ]; then
    echo "Detected Fedora..."
    dnf install -y make gcc gcc-c++ pkgconfig libX11-devel libXext-devel curl-devel zlib-devel freetype-devel libjpeg-turbo-devel openjpeg2-devel jbig2dec-devel
elif [ -f /etc/arch-release ]; then
    echo "Detected Arch Linux..."
    pacman -Syu --needed --noconfirm base-devel libx11 libxext curl zlib freetype2 libjpeg-turbo openjpeg2 jbig2dec
else
    echo "Unsupported distribution. Please install build-essential, X11, curl, freetype, and openjpeg manually."
fi
