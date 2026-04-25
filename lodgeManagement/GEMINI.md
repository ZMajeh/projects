# Lodge Management System (Web)

A web-based lodge management application built with Rust, Leptos, and Firebase.

## Project Overview

- **Frontend Framework:** [Leptos](https://leptos.dev/) (Client-Side Rendering)
- **Language:** Rust (compiled to WebAssembly)
- **Backend/Database:** Firebase (Authentication & Firestore)
- **Build Tool:** [Trunk](https://trunkrs.dev/)
- **Deployment:** Firebase Hosting

### Core Features

- **Room Management:** Track availability and status (Available, Occupied, Maintenance).
- **Customer Management:** Maintain customer records with Aadhaar verification.
- **Booking Management:** Handle check-ins, check-outs, and payments.
- **OCR Integration:** Extract Aadhaar details from images using Tesseract.js.
- **Camera Integration:** Capture customer photos directly from the browser.
- **Printable Bills:** Generate professional bills for customers.

## Project Structure

```text
.
├── GEMINI.md               # Project documentation
├── run.sh                  # Main runner script
├── requirements.sh         # Environment setup script
└── lodge-management-web/   # Main application directory
    ├── Cargo.toml          # Rust dependencies
    ├── index.html          # App shell & JS Bridge
    └── src/
        ├── main.rs         # Entry point & Routing
        ├── api.rs          # JS Bridge (wasm-bindgen)
        ├── models.rs       # Shared data structures
        ├── utils.rs        # Helpers (validation, state)
        └── components/     # UI Components
            ├── dashboard/  # Layout, Home, Bills
            ├── login.rs    # Auth UI
            ├── rooms.rs    # Room management
            ├── customers.rs # Customer records & OCR
            └── bookings.rs  # Booking management
```

## Architecture

The project follows a hybrid architecture where the UI and business logic are in Rust (Leptos), while external service integrations (Firebase, Camera, OCR) are handled via a JavaScript bridge.

### Core Files

- `src/main.rs`: High-level application state (Auth) and routing.
- `src/api.rs`: `wasm-bindgen` declarations for interop with JavaScript functions.
- `src/models.rs`: Defines `User`, `Room`, `Customer`, and `Booking` structs with Serde.
- `index.html`: Contains the CSS, Firebase SDK initialization, and the "JS Bridge" (window-scoped functions for Firestore, Camera, and OCR).

### JS Bridge (Interoperability)

New external features should be added as `window.functionName` in `index.html` and then declared in `src/api.rs`.

- **Auth:** `loginUser`, `signOutUser`
- **Firestore:** CRUD for `rooms`, `customers`, and `bookings`.
- **Media:** `startCamera`, `takeSnapshot`, `stopCamera`.
- **OCR:** `extractAadhaar` (using Tesseract.js).
- **Utility:** `manualVerifyAadhaar` (clipboard + external link).

## Building and Running

### Commands

Use the `run.sh` script from the project root.

- **Setup:** Installs Rust, `wasm32` target, Trunk, and Firebase Tools.
  ```bash
  ./run.sh setup
  ```

- **Development:** Starts hot-reloading server at `http://localhost:8080`.
  ```bash
  ./run.sh serve
  ```

- **Build:** Production-ready WebAssembly bundle in `lodge-management-web/dist`.
  ```bash
  ./run.sh build
  ```

- **Deployment:**
  ```bash
  cd lodge-management-web
  firebase deploy
  ```

## Development Conventions

- **State:** Use Leptos signals (`create_signal`) for local state. Prefer passing signals/callbacks to children.
- **Errors:** Handle JS Bridge failures using the `Result<JsValue, JsValue>` return type in `api.rs`.
- **Styling:** Vanilla CSS in `index.html`. Follow the existing color palette (defined in `:root`).
- **Naming:**
  - Rust: `snake_case` (functions/variables), `PascalCase` (structs/components).
  - Firestore/JS: `camelCase` for fields and bridge functions.
