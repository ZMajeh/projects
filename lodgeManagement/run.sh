#!/bin/bash

# Lodge Management System - Runner Script

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}Lodge Management System - Build & Run Manager${NC}"

# Function to check prerequisites
check_prereqs() {
    echo -e "${BLUE}Checking prerequisites...${NC}"
    
    if ! command -v rustup &> /dev/null; then
        echo -e "${RED}Rust is not installed. Please run './run.sh setup' first.${NC}"
        return 1
    fi

    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        echo -e "${BLUE}Installing wasm32-unknown-unknown target...${NC}"
        rustup target add wasm32-unknown-unknown
    fi

    if ! command -v trunk &> /dev/null; then
        echo -e "${RED}Trunk is not installed. Please run './run.sh setup' first.${NC}"
        return 1
    fi

    return 0
}

# Function to run setup
setup() {
    echo -e "${BLUE}Running requirements.sh...${NC}"
    chmod +x requirements.sh
    ./requirements.sh
}

# Function to start development server
serve() {
    if check_prereqs; then
        echo -e "${GREEN}Starting development server...${NC}"
        cd lodge-management-web
        trunk serve
    fi
}

# Function to build for production
build() {
    if check_prereqs; then
        echo -e "${GREEN}Building for production...${NC}"
        cd lodge-management-web
        trunk build --release
        echo -e "${GREEN}Build complete! Output is in lodge-management-web/dist${NC}"
    fi
}

# Main logic
case "$1" in
    setup)
        setup
        ;;
    serve|run)
        serve
        ;;
    build)
        build
        ;;
    *)
        echo "Usage: $0 {setup|serve|build}"
        echo ""
        echo "  setup  : Install Rust, Trunk, and other dependencies"
        echo "  serve  : Start the local development server (alias: run)"
        echo "  build  : Build the project for production"
        echo ""
        if [ -z "$1" ]; then
            echo -e "${BLUE}No command provided. Starting development server by default...${NC}"
            serve
        fi
        ;;
esac
