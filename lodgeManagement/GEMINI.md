# Lodge Management System (Web)

A web-based lodge management application built with Rust, Leptos, and Firebase.

## Project Overview

- **Frontend Framework:** [Leptos](https://leptos.dev/) (Client-Side Rendering)
- **Language:** Rust (compiled to WebAssembly)
- **Backend/Database:** Firebase (Authentication & Firestore)
- **Build Tool:** [Trunk](https://trunkrs.dev/)
- **Deployment:** Firebase Hosting

### Core Features

- **Room Management:** Track availability and status (Available, Occupied, etc.).
- **Customer Management:** Maintain customer records with Aadhaar verification.
- **Booking Management:** Handle check-ins, check-outs, and payments.
- **OCR Integration:** Extract Aadhaar details from images using Tesseract.js.
- **Camera Integration:** Capture customer photos directly from the browser.

## Architecture

The project follows a hybrid architecture where the UI and business logic are in Rust, while the external service integrations (Firebase, Camera, OCR) are handled via a JavaScript bridge.

- `src/main.rs`: Application entry point, routing, and high-level state management.
- `src/api.rs`: `wasm-bindgen` declarations for the JavaScript bridge functions defined in `index.html`.
- `src/models.rs`: Shared data structures (User, Room, Customer, Booking).
- `src/components/`: Modular UI components built with Leptos.
- `src/utils.rs`: Helper functions for validation, age calculation, and local storage.
- `index.html`: Contains the application shell, CSS styles, and the "JS Bridge" (Firebase SDK initialization and interop functions).

## Building and Running

### Commands

The project includes a `run.sh` script to simplify common tasks.

- **Setup Dependencies:**
  Installs Rust, Trunk, and other necessary tools.
  ```bash
  ./run.sh setup
  ```

- **Development Server:**
  Starts the Trunk development server with hot-reloading.
  ```bash
  ./run.sh serve
  ```

- **Build for Production:**
  Compiles the Rust code to WebAssembly and bundles assets.
  ```bash
  ./run.sh
  ```

- **Deployment:**
  Deploys the production build to Firebase Hosting.
  ```bash
  cd lodge-management-web
  firebase deploy
  ```

### Manual Prerequisites (if not using setup)

- [Rust](https://www.rust-lang.org/) (stable)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install --locked trunk`

## Development Conventions

- **State Management:** Use Leptos signals (`create_signal`) for local component state and global state where appropriate.
- **Interoperability:** New Firebase or external JS features should be added to the `<script>` tag in `index.html` and then declared in `src/api.rs`.
- **Styling:** Vanilla CSS is used within `index.html`. For component-specific styles, prefer adding them to the global stylesheet or using inline styles if necessary.
- **Naming:** Follow Rust naming conventions for backend logic and camelCase for JavaScript bridge functions as they appear in Firestore.
