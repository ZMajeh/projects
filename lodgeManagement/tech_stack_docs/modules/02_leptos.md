# Technology: Leptos (v0.6)

**Leptos** is a cutting-edge, high-performance web framework for Rust that leverages fine-grained reactivity.

## How It Helps the Lodge Management System
* **Fine-Grained Reactivity**: Unlike traditional frontend frameworks (like React) that use a Virtual DOM and re-render entire component trees upon state change, Leptos uses signals (`create_signal`). State changes trigger surgically targeted updates to specific DOM nodes directly. This produces an exceptionally responsive UI, even on mobile devices.
* **Declarative UI via `view!` Macro**: Leptos provides a familiar JSX-like syntax inside Rust using the `view!` macro. Developers can build modular, expressive UIs directly inside type-safe Rust code.
* **Client-Side Rendering (CSR) Paradigm**: Under CSR, the entire application compiles to WebAssembly and runs completely inside the user's browser. This matches the project's requirement of being highly interactive and responsive, minimizing network roundtrips for UI updates.

## Specific Role in this Project
Leptos is the foundation for the visual interface of the app. It constructs:
* **The Dashboard Layout**: The responsive sidebar, top headers, and routing outlets (`components/dashboard/`).
* **Interactive Views**: Sub-pages for managing Rooms, Bookings, Customers, Login, and User Whitelists.
* **Reactive State Orchestration**: Passing signals and callbacks between parent layouts and child modal views seamlessly to manage interactive forms (e.g., creating a booking, starting the camera, capturing an image, cropping pictures).
