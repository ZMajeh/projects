# Majeh's Email Browserr

A high-performance email browsing application built with Rust, Leptos, and Firebase.

## Architecture
- **Frontend Framework:** [Leptos](https://leptos.dev/) (Client-Side Rendering)
- **Language:** Rust (compiled to WebAssembly)
- **Build Tool:** [Trunk](https://trunkrs.dev/)
- **Backend/Database:** Firebase (Authentication & Firestore) + Gmail/Outlook REST APIs
- **Deployment:** Firebase Hosting

## Conventions & Standards
- **Naming:** Use snake_case for Rust functions and variables, PascalCase for Leptos components.
- **Styling:** Vanilla CSS for maximum flexibility and performance.
- **Error Handling:** Use `Result` and `Option` types rigorously; provide user-friendly error messages in the UI.
- **Components:** Keep components modular and state-driven using Leptos signals.

## Implementation Plan

### Phase 1: Initialization & "Hello World"
1. **Project Setup:** Initialize `Cargo.toml`, `Trunk.toml`, and `index.html`.
2. **UI Skeleton:** Create a 3-pane layout (Sidebar, Thread List, Viewport).
3. **CI/CD:** Set up GitHub Actions for auto-compilation and Firebase deployment.

### Phase 2: Authentication & Connectivity
1. **Firebase Auth:** Integrate Google/Microsoft OAuth2 for email access.
2. **API Integration:** Fetch real email headers using `gloo-net` and the Gmail/Outlook APIs.

### Phase 3: Features & Polish
1. **Email Reading:** Full body rendering with HTML sanitization.
2. **Actions:** Support Archive, Delete, and Reply.
3. **Aesthetics:** Apply "Glassmorphism" theme with CSS.

## Deployment Workflow
- Every push to `main` triggers a GitHub Action.
- The Action builds the WASM binary using `trunk build --release`.
- The resulting `dist/` directory is deployed to Firebase Hosting.
