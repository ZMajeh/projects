# Tutorial: Trunk for Asset Pipeline

**Trunk** is the bundler for our WASM application.

## Setup
Install Trunk globally:
```bash
cargo install trunk
```

## Running
Use the provided `run.sh` script, which wraps Trunk commands:
```bash
# Development
./run.sh serve

# Build for production
./run.sh build
```

## Configuration
Trunk uses `index.html` as the entry point. It automatically detects and processes:
- `<link rel="rust" href="Cargo.toml" />`
- Stylesheets (`<link rel="stylesheet" href="..." />`)
- Images and other assets.
