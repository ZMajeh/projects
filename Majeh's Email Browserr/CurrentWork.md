# Current Work — Majeh's Email Browserr

## Project Goal (per user requirements)
- Support **90+ email accounts** via **OAuth (Google/Gmail)**
- Use **refresh tokens** so users never need to re-login (tokens don't expire)
- Store OAuth tokens in **browser localStorage**, encrypted with a user password
- **Export/Import** vault to file, secured through password
- Connected accounts appear in the **sidebar**; clicking one fetches and shows its emails

## Build Status
- `cargo check` — **PASSING** (zero errors; only pre-existing `vault_name` dead-code warning)
- `cargo check --target wasm32-unknown-unknown` — **PASSING** (zero errors)
- `trunk build` — **PASSING** (dev; `dist/logo.png` now present ✅)
- `trunk build --release` — verified this session

---

## Completed ✅

### OAuth PKCE Flow (`src/oauth.rs`)
1. `config()` — reads `client_id`, `client_secret` from localStorage (fallback to meta tag), derives `redirect_uri`
2. `start_google_oauth()` — PKCE `state` + `verifier` in sessionStorage, `access_type=offline&prompt=consent`
3. `handle_google_callback()` — validates `state`, exchanges code, fetches userinfo
4. `extract_google_tokens()`; `refresh_access_token()` now actively used by token auto-refresh
5. **Gmail API functions**:
   - `gmail_list_messages_query(access_token, q)` — always scoped to `labelIds=INBOX&maxResults=20`, optional Gmail search query (e.g. `after:<epoch_s> before:<epoch_s>` for date-range fetches)
   - `gmail_get_message()` — full message incl. `internalDate`, `mimeType`, headers, parts
   - `decode_b64url()` — Gmail base64url body decoding (padding-tolerant)
   - `gmail_extract_body()` — recursive `text/plain` extraction from `payload.parts` (skips HTML/attachments, snippet fallback)
   - `parse_address_header()` — `Jane Doe <jane@x.com>` → (display name, address)

### Token Vault (`src/tokens.rs`)
- `TokenVault` component with unlock/lock/save/export/import
- Vault encrypted with `aes-gcm` + `pbkdf2`, stored in `localStorage`
- Export/import secured through password
- `OAuthToken` struct: `provider`, `email`, `access_token`, `refresh_token`, `expires_at`
- Pending OAuth token auto-persistence from callback flow
- **Shared vault session (uncommitted, in working tree)**: `vault_password()` (thread-local `Option<String>`) + `vault_epoch()` (reactive bump) + `vault_is_unlocked()`. Unlocking once (Splash *or* TokenVault) unlocks everywhere — no second prompt, and `save_vault()` always uses the real password. Replaced an earlier `OnceLock<RwSignal>` design that panicked with "already been disposed" because the signal was created outside any reactive owner.
- **Single persistence choke point**: `save_vault(password, tokens)`; `persist_vault_now(tokens)` re-encrypts with the session password (no-op while locked). Called after *every* token refresh so a page reload restores fresh tokens instead of yesterday's copy.
- `lock()` clears in-memory state only — the encrypted vault stays in localStorage (the old code *deleted* the vault on lock, which made accounts "vanish").

### Fetch Pipeline (`src/app.rs`)
- **Per-account email cache** (`email_cache: HashMap<account_email, Vec<Email>>`) — re-clicking a fetched account shows emails **instantly, no refetch**
- **Fetch tasks** (`FetchTask` + `TaskStatus` in `data.rs`) — one task per account(+date-range) with live `fetched/total` progress; duplicate-running guard; auto-hide when done
- **Token auto-refresh** — `ensure_fresh_token()` refreshes 60s before expiry and writes the fresh token back into `connected_accounts` + `active_account` (session level)
- **Cache merge with dedupe by Gmail `msg_id`** — date-range fetches extend the cache without duplicates; sorted by `internalDate` desc
- **Star/read write-through** — guarded, untracked `create_effect` syncs selection/star toggles back into the active account's cache
- `gmail_msg_to_email()` — Subject/From/**To** headers, `internalDate` → formatted time, full body → paragraphs, `labelIds` → folder/unread/starred
- Account switch resets selection + folder (no stale reading pane)
- Helpers: `date_input_to_ms()` (local midnight from `<input type=date>`), `sleep()` via JS timer (no new crate)

### Token Refresh — Bug Fix (client_secret)
- **Bug**: Google's token endpoint rejected refreshes with `invalid_request: "client_secret is missing."` — `refresh_access_token()` sent only `client_id` + `refresh_token`
- **Fix**: `refresh_access_token(client_id, client_secret, refresh_token)` now includes `client_secret` — the same secret stored in **Settings → Google Client Secret** (`localStorage.oauth_client_secret`, read via `config()`)
- Guard added: empty secret fails fast with "save it in Settings → Google Client Secret" instead of a cryptic Google error
- All 3 call sites updated: `ensure_fresh_token()` (app.rs) + per-row and bulk refresh (tokens_page.rs)

### Vault Unlock During Splash (bug fix — hard gate)
- **Bug**: `TokenVault` only lived in the Settings tab, so `connected_accounts` stayed empty until the user manually unlocked there. No accounts showed in the sidebar, so no emails could be fetched without a trip to Settings first.
- **Fix**: The splash screen is now a **hard gate** — it stays on screen until the user either:
  - **Unlocks an existing vault** (enters password → `connected_accounts` populated immediately), or
  - **Creates a new vault password** (first run → creates encrypted empty vault)
- Only after success does the splash dismiss and the app appear.
- Extracted `unlock_vault(password)` + `create_vault(password)` + `vault_exists()` as public functions in `src/tokens.rs`; `VaultPayload` is `pub`.
- Removed the 3-second auto-dismiss timer; splash now waits indefinitely for user action.

### Splash Screen (`src/components/splash.rs`)
- Hard-gate splash: centered `logo.png`, app name, indefinite progress bar
- When a vault exists: password input + "Unlock" button + error display
- When no vault: "Create your vault password" (new + confirm fields)
- Unlocking/creating dismisses the splash and reveals the app


### Tokens Page (`src/components/tokens_page.rs`) — new tab
- Reachable from the sidebar footer ("Tokens") and the bottom nav (4th tab)
- One row per connected account: **email + provider, full access token, full refresh token (or "— none —"), status, action**
- **Status**: "Active — expires in 3h 20m" (green) / "Expired 12m ago" (red, row highlighted) / "No expiry info" (gray) — computed live from `expires_at`
- **Per-row "Refresh" button** — spins while running, calls Google's refresh endpoint, updates the token + expiry in the shared account list **and persists to the encrypted vault** (`persist_vault_now`); disabled when already refreshing or no refresh token
- **Top "Refresh all expired" button** — sequentially refreshes every expired token with per-step status ("Refreshing 2/3 (x@y.com)…"), persists each success immediately, and a final "Done — refreshed N of M"
- **Copy buttons** for both tokens (system clipboard)
- **Email backup download/upload** — download serializes the per-account email cache to `majeh-emails-backup.json`; upload reads a `.json` file via `FileReader`, merges into the cache by account, and re-saves. Fixed for Leptos 0.6 (`event_target::<HtmlInputElement>` + `files().get(0)` + `Closure::wrap`; added `File`/`FileList` web-sys features).
- Summary header ("N connected — A active, E expired, U unknown"), empty state, responsive single-column layout under 900px

### Fetch Panel (`src/components/fetch_panel.rs`)
- Progress bars: **"Fetching 20 emails for x@gmail.com — 12/20"** (indeterminate bar while listing; green + auto-hide when done)
- Collapsible **Fetch by date** controls: From / To date inputs + account select (single account or all) + "Fetch Range"
- Each **account + date combination** gets its own task and progress bar (queue several ranges by clicking again)
- User-facing **error banner** with dismiss (medium-priority "Error UI" item)

### Sidebar (`src/components/sidebar.rs`)
- Mock + connected OAuth accounts ("OAuth" badge); clicking an OAuth account switches to Mailbox and loads/fetches its emails
- **Per-account fetched-email count** ("142 emails") shown under the address, driven by the cache
- Removed unused `loading` prop

### Reading Pane (`src/components/reading_pane.rs`)
- **Real `To` field** from the Gmail `To` header ("To me" only as empty fallback)
- Sender shows `Name <addr>` only when they differ (no duplicate `x <x>`)

### Settings UI (`src/components/settings.rs`)
- "Google Client Secret" input with Save/Change; stored in `localStorage.oauth_client_secret` (never committed)
- **Leptos reactivity fix (uncommitted)**: `secret_saved` / `secret_error` reads inside `view!` wrapped in `move ||` closures (lines ~135/146/161) — previously read signals outside a tracking context, causing "signal accessed outside reactive tracking context" console warnings.

### OAuth Callback (`src/app.rs`)
- After a successful token exchange, the single-use `?code=`/`state=` query is stripped from the URL via `history.replace_state` — a later browser refresh can no longer re-trigger the callback and fail with `state mismatch` on the already-consumed auth code.

### Data (`src/data.rs`)
- `Email` gained `msg_id`, `to_field`, `timestamp_ms`; `next_email_id()` (session-unique ids); `format_email_time()` (local "10:24 AM" / "Yesterday" / "Mar 8" / "3/8/24")
- `FetchTask` + `TaskStatus` with `progress_text()` ("Fetching 20 emails for x — 12/20") and `fraction()`

### Logo (`index.html`)
- **Fix**: `logo.png` was referenced by CSS but never copied to `dist/` (404) — added `<link data-trunk rel="copy-file" href="logo.png"/>`; verified `dist/logo.png` after build

---

## In Progress 🔄
- **Vault persistence hardening (uncommitted, in working tree — `src/tokens.rs`, `src/app.rs`, `src/components/settings.rs`, `src/components/splash.rs`, `src/components/tokens_page.rs`)**
  - Shared session (`vault_password` / `vault_epoch`), `save_vault` choke point, `persist_vault_now` on all refresh paths, non-destructive `lock()`, reactive settings fix, OAuth URL cleanup
  - `cargo check --target wasm32-unknown-unknown` — PASSING (zero errors; only 3 pre-existing warnings)
  - Still to do: `trunk build` + browser re-test (second-account add, reload-keeps-fresh-tokens, no "already been disposed" panic), then commit

---

## Todo Next 📋

### High Priority (blocking core UX)
1. ~~Progress bar during email fetch~~ ✅ FetchPanel + `FetchTask`
2. ~~Date/time of fetched emails~~ ✅ `internalDate` → `format_email_time()`
3. ~~To field always shows "me"~~ ✅ parsed from `payload.headers`
4. ~~Email fetch caching~~ ✅ per-account cache, instant re-click
5. ~~Email body not completely fetched~~ ✅ `gmail_extract_body()` plain-text recursion
6. ~~App logo not visible~~ ✅ Trunk `copy-file` → `dist/logo.png`
7. ~~Per-account email count~~ ✅ sidebar "N emails"
8. ~~Manual date/time filter with per-combination progress~~ ✅ Fetch by date

### Medium Priority
7. **Email-wise Mail/Labels subsections** — clicking an email expands:
   - **Mail subsection** — emails from/to that specific address
   - **Labels subsection** — labels on that email's thread
   - Global Mail and Labels sections remain (all emails across all accounts)
8. ~~Account switching without stale data~~ ✅ selection/folder reset + cache
9. ~~Token auto-refresh~~ ✅ session level (see Known Limitations)
10. ~~Error UI~~ ✅ dismissible fetch-error banner

### Low Priority
11. **Inbox refresh action** — cache never expires and re-click never refetches by design; add a manual "Refresh" button (or TTL) to pick up new mail
12. **Outlook support (Phase 2)** — also clears the `vault_name` warning
13. ~~Persist refreshed access tokens to the vault~~ ✅ `persist_vault_now()` on all 3 refresh paths (per-row, refresh-all, `ensure_fresh_token`); reload restores fresh tokens
14. **HTML-only emails** — body falls back to `snippet` when no `text/plain` part exists (consider a minimal HTML→text)
15. **Page beyond 20 messages** — `maxResults=20`; `nextPageToken` parsed but unused (pagination)
16. **Security** — `index.html` no longer contains client_secret (reads from localStorage at runtime)
17. ~~Splash screen~~ ✅ 3s logo + indefinite progress bar (login intentionally skipped — see Splash section)

---

## Known Limitations (non-blocking)
- Default fetch returns the 20 most recent INBOX messages (`labelIds=INBOX`)
- `logo.png` is 1.9 MB for a 32px icon — worth optimizing later

## Known Warnings (non-blocking)
- `vault_name` never used (Phase 2 — Outlook support)
- `.clone()` on `&str` in `reading_pane.rs` (pre-existing)
- `HashMap` unused import in `app.rs`, `JsCast` unused import in `data.rs` (pre-existing, warned not denied)

---

## 🔐 Secrets Management — Options for Firebase Hosting

Since Firebase Hosting serves **static files only**, there is no server-side environment to hide secrets. Below are all viable approaches for handling the Google OAuth `client_secret`, ranked from most to least recommended for this project.

### Option A: Browser localStorage (Current Implementation)

**How it works:** The user enters the client secret in Settings → it's saved to `localStorage.oauth_client_secret` → `config()` reads it at runtime.

**Pros:**
- Nothing committed to git
- No build/CI changes needed
- Works immediately on any deployment
- Secret lives only in the user's browser

**Cons:**
- User must enter it once per browser/device
- Visible to anyone with access to the browser's DevTools
- Not synced across devices

**Implementation:** Already done in `src/components/settings.rs` + `src/oauth.rs` `config()`.

---

### Option B: Build-Time Substitution (CI/CD)

**How it works:** GitHub Actions replaces a placeholder in `index.html` with the secret from GitHub Secrets during the build.

**`index.html`:**
```html
<meta id="__oauth_client_id" data-client-id="..." data-client-secret="__GOOGLE_CLIENT_SECRET__"/>
```

**`.github/workflows/deploy.yml`:**
```yaml
- name: Build
  run: |
    sed -i 's|data-client-secret="__GOOGLE_CLIENT_SECRET__"|data-client-secret="${{ secrets.GOOGLE_CLIENT_SECRET }}"|' index.html
    trunk build --release
- name: Deploy
  uses: FirebaseExtended/action-hosting-deploy@v0
```

**Pros:**
- Zero runtime code needed
- Works for all users automatically
- Secret never touches the user's browser DevTools (it's in the served HTML, but still viewable via View Source)

**Cons:**
- Secret is embedded in the built HTML — anyone can view source and see it
- Requires GitHub Actions setup
- Committing built artifacts with secrets is risky

**Verdict:** Not recommended for client-side secrets since View Source exposes them anyway.

---

### Option C: Runtime Config File (`config.json`)

**How it works:** A separate `config.json` file (gitignored, deployed separately) is fetched at runtime.

**`config.json`** (gitignored):
```json
{
  "googleClientSecret": "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"
}
```

**`firebase.json`:**
```json
{
  "hosting": {
    "public": "dist",
    "headers": [
      {
        "source": "/config.json",
        "headers": [
          {"key": "Cache-Control", "value": "no-cache, no-store, must-revalidate"}
        ]
      }
    ]
  }
}
```

**In `src/oauth.rs` `config()`:**
```rust
let config_resp = gloo_net::http::Request::get("/config.json")
    .send()
    .await
    .ok()
    .and_then(|r| r.json::<serde_json::Value>().ok());
let client_secret = config_resp
    .and_then(|v| v.get("googleClientSecret"))
    .and_then(|v| v.as_str())
    .unwrap_or("");
```

**Pros:**
- Secret not in `index.html` or git
- Can be updated without rebuilding
- Deployed separately via Firebase CLI

**Cons:**
- Extra HTTP request on app load
- Still visible in browser DevTools (Network tab)
- Requires separate deployment step

---

### Option D: Firebase Remote Config

**How it works:** Store the secret as a Remote Config parameter; fetch it at runtime via the Firebase JS SDK.

**JavaScript (called from Rust via wasm-bindgen):**
```javascript
const remoteConfig = firebase.remoteConfig();
remoteConfig.fetchAndActivate().then(() => {
    const secret = remoteConfig.getValue('googleClientSecret').asString();
    localStorage.setItem('oauth_client_secret', secret);
});
```

**Pros:**
- Managed in Firebase Console (no code changes to update)
- Can target different environments (dev/prod)
- Firebase handles caching and rollout

**Cons:**
- Requires Firebase SDK setup
- Still visible in DevTools
- Extra dependency

---

### Option E: Backend Proxy (Most Secure)

**How it works:** Instead of the browser calling Google's token endpoint directly, the browser calls your own backend (Firebase Functions, Cloud Run, etc.) which holds the secret and forwards the request to Google.

**Flow:**
1. Browser sends `code` to your backend
2. Backend adds `client_secret` and calls Google's token endpoint
3. Backend returns tokens to browser

**Pros:**
- Secret never touches the browser
- Most secure option
- Can add rate limiting, logging, etc.

**Cons:**
- Requires building and hosting a backend
- More complex architecture
- Costs money (Firebase Functions, etc.)
- CORS configuration needed

---

### Recommendation for This Project

| Priority | Approach | Reason |
|----------|----------|--------|
| **Now** | **Option A** (localStorage) | Already implemented, simplest, works immediately |
| **Later** | **Option C** (config.json) | Better separation, no UI needed |
| **If security critical** | **Option E** (backend proxy) | Secret never reaches browser |

### ⚠️ Important Security Reality

For **client-side web OAuth**, the `client_secret` is inherently visible somewhere in the client (HTML, JS, or network traffic). Google designed web OAuth this way — the secret authorizes the token exchange but does not by itself grant account access. The real protection is:
- Redirect URIs are registered in Google Cloud Console
- Refresh tokens are bound to the specific client
- Users can revoke access in their Google Account settings

For a personal email browser with 90 accounts, **Option A (localStorage)** is the pragmatic choice.

---

## Git Status
- Branch: `main`
- Last commit: `d2681b72` "Fetch email option added" (email backup download/upload + Leptos 0.6 fix)
- **Uncommitted changes (this session, all in working tree):**
  - `src/tokens.rs` — shared vault session (`vault_password`/`vault_epoch`/`vault_is_unlocked`), `save_vault` choke point, `persist_vault_now`, non-destructive `lock()`, pending-OAuth effect uses session password
  - `src/app.rs` — OAuth URL cleanup (`replace_state`), `persist_vault_now` after `ensure_fresh_token`
  - `src/components/splash.rs` — session published via `unlock_vault`/`create_vault`
  - `src/components/settings.rs` — reactive `move ||` fixes for `secret_saved`/`secret_error`
  - `src/components/tokens_page.rs` — `persist_vault_now` on per-row + refresh-all paths
  - `CurrentWork.md` — this update
- Build artifacts (`dist/`, `target/`) untracked
