#!/bin/bash

# Function to install system dependencies based on package manager
install_sys_deps() {
    if command -v apt-get &> /dev/null; then
        echo "Detected Debian-based system (apt)"
        sudo apt-get update
        sudo apt-get install -y curl build-essential
    elif command -v apk &> /dev/null; then
        echo "Detected Alpine-based system (apk)"
        apk add --no-cache curl build-base
    elif command -v dnf &> /dev/null; then
        echo "Detected RHEL-based system (dnf)"
        sudo dnf groupinstall -y "Development Tools"
        sudo dnf install -y curl
    elif command -v pacman &> /dev/null; then
        echo "Detected Arch-based system (pacman)"
        sudo pacman -Sy --noconfirm curl base-devel
    else
        echo "Warning: Could not detect package manager. Please ensure 'curl' and build tools are installed manually."
    fi
}

install_sys_deps

# Update PATH for the current script execution
export PATH="$HOME/.cargo/bin:$PATH"

echo "Checking for Rust..."
if ! command -v rustup &> /dev/null; then
    echo "Rustup not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rustup is already installed."
fi

echo "Ensuring wasm32-unknown-unknown target is installed..."
rustup target add wasm32-unknown-unknown

echo "Checking for Trunk..."
if ! command -v trunk &> /dev/null; then
    echo "Installing Trunk (this may take a few minutes)..."
    cargo install --locked trunk
else
    echo "Trunk is already installed."
fi

echo "Installing Firebase Tools via npm..."
# Attempt global install, fallback to local if it fails (e.g., due to permissions)
if command -v npm &> /dev/null; then
    sudo npm install -g firebase-tools || {
        echo "Global npm install failed. Attempting local install..."
        npm install firebase-tools
    }
else
    echo "Warning: npm not found. Please install Node.js and npm to use Firebase Tools."
fi

echo "--------------------------------------------------"
echo "Setup complete!"
echo "IMPORTANT: Please run 'source \$HOME/.cargo/env' or restart your terminal to update your PATH."
echo "--------------------------------------------------"
