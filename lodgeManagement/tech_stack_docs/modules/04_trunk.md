# Technology: Trunk

**Trunk** is a WASM web application bundler for Rust.

## How It Helps the Lodge Management System
* **Zero-Config Simplicity**: Trunk serves as the build and assets pipeline, managing CSS style sheets, image assets, manifest files, and the compilation of Rust code into WASM without requiring a complex Webpack or Vite configuration.
* **Hot Reloading & Fast Feedback**: During development, Trunk watches source directories, recompiles Rust binaries, and injects hot-reloaded code into the browser, keeping the feedback loop tight.
* **Seamless Static Packaging**: Trunk bundles the final application into an index file, `.wasm` binary, and asset package in `dist/` ready for serverless hosting.

## Specific Role in this Project
Trunk coordinates the build pipeline:
* Reads `index.html` as the source blueprint.
* Pulls `Cargo.toml` and builds the main Rust compilation target into WASM.
* Processes static styles, assets, and service workers (`sw.js`).
* Deploys output into the static directory compiled for Firebase Hosting.
