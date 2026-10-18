# Current Work — Majeh's Email Browserr

## Build Status
- `cargo check` — **PASSING** (zero errors)
- `cargo check --target wasm32-unknown-unknown` — **PASSING** (zero errors)
- `trunk build` — not yet verified after latest edits

---

## Completed ✅

### OAuth PKCE Flow (`src/oauth.rs`)
1. `config()` — derives redirect URI from `window.location.origin() + "/auth/callback/google"` (works for both `http://localhost:8080` dev and `https://majehs-email-browserr.web.app` prod)
2. `start_google_oauth()` — returns `Result<(), String>`, generates PKCE `state` + `verifier`, stores them in `sessionStorage`, computes `code_challenge(&verifier)`, builds consent URL, navigates via `window.location().set_href()`
3. `handle_google_callback()` — reads PKCE verifier from `sessionStorage` **before** clearing it (was a critical bug: delete-then-read), validates `state` against CSRF, exchanges code for tokens, fetches Google userinfo, returns `(OAuthToken, UserInfo)`
4. `extract_google_tokens()` — uses `js_sys::Date::now()` (WASM-safe) instead of `std::time::SystemTime`, fixed `.cloned()` on `Option<&str>`
5. Added `use wasm_bindgen::JsCast`, fixed `JsValue` formatting (`{e:?}`), fixed `.attr()` → `.get_attribute()`

### App Callback Handling (`src/app.rs`)
- On mount, checks `window.location.href` for `?code=` or `/auth/callback/google`
- When callback detected: sets `active_tab` to `Tab::Settings` and calls `handle_google_callback()` via `spawn_local`
- Removed unused `leptos_router::prelude::*` import

### Token Vault (`src/tokens.rs`)
- Removed misplaced `extract_google_tokens()` function that was incorrectly inside the `view! { ... }` macro body

### Settings UI (`src/components/settings.rs`)
- "Connect Gmail" button now handles `Result<(), String>` return from `start_google_oauth()`

### Config / HTML (`index.html`, `Cargo.toml`)
- `index.html` — already has `<meta id="__oauth_client_id" data-client-id="..."/>`
- `Cargo.toml` — already has all required `web-sys` features

---

## In Progress 🔄
- Nothing currently in progress.

---

## Todo Next 📋

### High Priority
1. **Pending OAuth Token state** — After `handle_google_callback()` succeeds, the `OAuthToken` should be queued so it's automatically saved into the encrypted vault when the user unlocks/creates their vault password. Currently the token is logged but not persisted to the vault.
2. **UI feedback in Settings** — Show connected account status (email, provider, token expiry) in the Settings view after a successful OAuth connect.
3. **Test full OAuth flow end-to-end** — Navigate to the app, click "Connect Gmail", complete Google consent, verify redirect back and token exchange.

### Medium Priority
4. **Token auto-refresh** — Use `refresh_access_token()` when an access token is near expiry.
5. **Vault integration with connected accounts** — When a Gmail account is connected via OAuth, add it to the sidebar account list.
6. **Error UI** — Show user-facing error messages when OAuth fails (currently only console logs).

### Low Priority
7. **Outlook support (Phase 2)** — `vault_name()` and `refresh_access_token()` are placeholders for Phase 2.
8. **Trunk build verification** — Run `trunk build --release` to confirm the WASM bundle builds cleanly.

---

## Known Warnings (non-blocking)
- `vault_name` never used (Phase 2 — Outlook support)
- `refresh_access_token` never used (Phase 2 — token refresh)
- `.clone()` on `&str` in `reading_pane.rs` (pre-existing)

---

## Git Status
- Modified: `.gitignore`, `Cargo.toml`, `index.html`, `src/app.rs`, `src/oauth.rs`, `src/tokens.rs`, `src/components/settings.rs`, `src/main.rs`, `run.sh`, `Trunk.toml`
- Deleted: `src/components/mailbox.rs`, `dist/` artifacts
- New: `CurrentWork.md`