# Current Work — Majeh's Email Browserr

## Project Goal (per user requirements)
- Support **90+ email accounts** via **OAuth (Google/Gmail)**
- Use **refresh tokens** so users never need to re-login (tokens don't expire)
- Store OAuth tokens in **browser localStorage**, encrypted with a user password
- **Export/Import** vault to file, secured through password
- Connected accounts appear in the **sidebar**; clicking one fetches and shows its emails

## Build Status
- `cargo check` — **PASSING** (zero errors)
- `cargo check --target wasm32-unknown-unknown` — **PASSING** (zero errors)
- `trunk build` — not yet verified after latest edits

---

## Completed ✅

### OAuth PKCE Flow (`src/oauth.rs`)
1. `config()` — reads `client_id`, `client_secret` from localStorage (fallback to meta tag), derives `redirect_uri`
2. `start_google_oauth()` — generates PKCE `state` + `verifier`, stores in `sessionStorage`, computes `code_challenge(&verifier)`, builds consent URL with `access_type=offline`
3. `handle_google_callback()` — validates `state`, exchanges code for tokens (sends `client_secret`), fetches userinfo
4. `extract_google_tokens()` — WASM-safe timestamps
5. `refresh_access_token()` — available for token refresh
6. **Gmail API functions**: `gmail_list_messages()`, `gmail_get_message()` with full response models — **fixed camelCase serde renames** (`threadId`, `nextPageToken`, `labelIds`)

### Token Vault (`src/tokens.rs`)
- `TokenVault` component with unlock/lock/save/export/import
- Vault encrypted with `aes-gcm` + `pbkdf2`, stored in `localStorage`
- Export/import secured through password
- `OAuthToken` struct: `provider`, `email`, `access_token`, `refresh_token`, `expires_at`
- Pending OAuth token auto-persistence from callback flow

### App (`src/app.rs`)
- OAuth callback detection and handling
- `connected_accounts` shared signal passed to Sidebar and Settings
- `active_account` signal for tracking selected OAuth account
- `fetch_emails_for_account()` — calls Gmail API to list + fetch messages, converts to `Email` structs
- Loading state during email fetch

### Sidebar (`src/components/sidebar.rs`)
- Displays mock accounts + connected OAuth accounts (with "OAuth" badge)
- Clicking an OAuth account switches to Mailbox tab AND fetches emails via Gmail API
- Loading state available

### Settings UI (`src/components/settings.rs`)
- Added **"Google Client Secret"** input field with Save/Change buttons
- Secret stored in `localStorage.oauth_client_secret` (never committed)
- Auto-loads existing secret on mount
- Shows error messages for save failures

### Data (`src/data.rs`)
- `Email` and `Account` structs changed from `&'static str` to `String` for dynamic API data

---

## In Progress 🔄
- Nothing currently in progress.

---

## Todo Next 📋

### High Priority (blocking core UX)

1. **Progress bar during email fetch** — Show how many emails are being fetched and for which account (e.g., "Fetching 20 emails for mundhe.anand.anand@gmail.com — 12/20"). Currently there's only a loading state with no progress feedback.

2. **Date/time of fetched emails** — Gmail API returns `internalDate` (epoch ms) per message. Currently not parsed or displayed. Need to extract and format date/time in the email list and reading pane.

3. **To field always shows "me"** — The `To` header from Gmail API is not being parsed/displayed correctly. Currently reading pane shows "To me" hardcoded. Need to extract the actual `To` header from `payload.headers`.

4. **Email fetch caching** — Re-clicking an already-fetched account re-fetches all emails from Gmail API. Need to cache fetched emails per account (by email address) so re-clicking shows cached results instantly without re-fetching.

5. **Email body not completely fetched** — Gmail API `snippet` only shows partial body. Need to fetch full body from `payload.parts` or `payload.body.data` (base64url encoded) and populate `Email.body` completely.

6. **App logo not visible** — The app logo/brand image is not showing on the page. Need to verify the logo asset path and `brand-logo` div styling.

7. **Per-account email count** — Maintain a count of how many emails have been fetched for each connected email account (e.g., "Gmail Personal: 142 emails"). Display in sidebar next to each account.

8. **Manual date/time filter for fetching** — User can manually specify date/time ranges for which they want to fetch emails, supporting multiple accounts and multiple dates. Each account+date combination needs its own progress bar showing fetch status.

### Medium Priority

7. **Email-wise Mail/Labels subsections** — When clicking an email in the reading pane, it should expand and show 2 subsections:
   - **Mail subsection** — emails from/to that specific email address
   - **Labels subsection** — labels applied to that email's thread
   - Global Mail and Labels sections should remain and fetch from all emails across all accounts

8. **Account switching** — When clicking a different OAuth account, properly refresh emails without stale data mixing

9. **Token auto-refresh** — Use `refresh_access_token()` when access token is near expiry before Gmail API calls

10. **Error UI** — Show user-facing error messages when OAuth or Gmail API fails (currently only console logs)

### Low Priority

11. **Outlook support (Phase 2)**
12. **Trunk build verification** — Run `trunk build --release`
13. **Security** — `index.html` no longer contains client_secret (reads from localStorage at runtime)

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

## Known Warnings (non-blocking)
- `vault_name` never used (Phase 2 — Outlook support)
- `refresh_access_token` never used (will be used when token refresh is implemented)
- `.clone()` on `&str` in `reading_pane.rs` (pre-existing)

---

## Git Status
- Branch: `main` — pushed to `origin/main` ✅
- Last commit: `06e99035` "Fetch emails via Gmail API with OAuth PKCE"
- Modified: `.gitignore`, `Cargo.toml`, `index.html`, `src/app.rs`, `src/oauth.rs`, `src/tokens.rs`, `src/components/settings.rs`, `src/components/sidebar.rs`, `src/components/reading_pane.rs`, `src/data.rs`, `src/main.rs`, `run.sh`, `Trunk.toml`
- New: `CurrentWork.md`
- Build artifacts (`dist/`, `target/`) removed from tracking
