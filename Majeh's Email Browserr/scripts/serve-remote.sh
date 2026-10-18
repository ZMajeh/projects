#!/usr/bin/env bash
# Serve the app on all interfaces for remote (LAN) access.
# - trunk listens on 0.0.0.0:8080 (Trunk.toml [serve] address)
# - Prints the Windows-side sync instructions if port forwarding is needed
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v trunk >/dev/null 2>&1; then
    echo "trunk not found. Run './run.sh setup' first." >&2
    exit 1
fi

if grep -qi microsoft /proc/version; then
    WSL_IP=$(hostname -I | awk '{print $1}')
    WIN_SCRIPT=$(wslpath -w "$(realpath scripts/sync-portproxy.ps1)" 2>/dev/null || true)
    echo "==============================================="
    echo " WSL detected. This server: ${WSL_IP}:8080"
    echo " If .wslconfig uses mirrored networking (Win11),"
    echo "   remote URL = http://<Windows-LAN-IP>:8080"
    echo " Otherwise, one-time per boot, in ADMIN PowerShell:"
    if [ -n "${WIN_SCRIPT}" ]; then
        echo "   powershell -ExecutionPolicy Bypass -File \"$WIN_SCRIPT\""
        echo " (a Windows copy also lives at C:\\Users\\mundh\\sync-portproxy.ps1)"
    else
        echo "   see scripts/sync-portproxy.ps1"
    fi
    echo "==============================================="
fi

exec trunk serve
