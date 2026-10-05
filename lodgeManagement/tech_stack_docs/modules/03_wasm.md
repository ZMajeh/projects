# Technology: WebAssembly (WASM) & `wasm-bindgen`

**WebAssembly** is a binary instruction format for a stack-based virtual machine, designed to run in browsers at near-native speed. **`wasm-bindgen`** is the glue library that facilitates high-level interoperation between Rust and JavaScript.

## How It Helps the Lodge Management System
* **Bridging the Ecosystems**: Rust's ecosystem is vast, but browser APIs (such as Web Camera access, Google Drive client SDKs, Firestore Client libraries, and Tesseract.js) are predominantly written in JavaScript. `wasm-bindgen` allows the Rust-based Leptos application to import JavaScript functions and call them natively as Rust functions, and vice-versa.
* **Zero-Overhead Bindings**: `wasm-bindgen` compiles bindings to highly efficient instructions, enabling fast serialization/deserialization of complex structs (e.g., passing customer profiles or booking records) across the boundary using `serde-wasm-bindgen`.

## Specific Role in this Project
Defined in `src/api.rs`, this boundary allows Rust components to:
* Trigger Firebase Auth actions (`login_user`, `sign_out_user`).
* Perform CRUD transactions on Firestore collections.
* Stream media feeds from the camera and trigger snapshots.
* Request text recognition (OCR) on base64 Aadhaar scans.
