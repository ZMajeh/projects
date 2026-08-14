# Tutorial: Firebase Setup & Integration

This tutorial covers setting up Firebase for the Lodge Management application.

## Prerequisites
1. A Google Account.
2. Firebase account (project set up).

## Setup
1. Create a Firebase project.
2. Initialize Firebase in the `lodge-management-web` directory:
   ```bash
   firebase init
   ```
3. Configure Firestore and Authentication in the Firebase Console.

## Integration
The project interacts with Firebase via the JS Bridge defined in `index.html` and `src/api.rs`.

Example of calling Firebase from Rust (via JS Bridge):
```rust
// src/api.rs
#[wasm_bindgen]
extern "C" {
    pub fn loginUser();
}
```
In your Rust code:
```rust
api::loginUser();
```
Ensure `loginUser` is implemented in `index.html` to call `firebase.auth().signInWithPopup(...)`.
