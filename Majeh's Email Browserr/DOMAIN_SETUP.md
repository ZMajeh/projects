# Owning Your Domain (required by the Google Play "App Homepage" review)

## Why this matters

Google Play's *App Homepage* guidance says:

> Your homepage and privacy policy should not be hosted on a third-party platform
> where you can't verify that you own your subdomain.

The current home is `https://majehsemailviewer.web.app`. `web.app` is a **shared
Google/Firebase domain** — the project owns the subdomain, but the registrable
domain belongs to Google. That is the one item on the checklist that is still
exposed. Everything else (purpose, functionality, data transparency, privacy
link, no-login visibility) is implemented in the app.

**Fix:** point the app at a domain you registered yourself (e.g.
`majehsemailviewer.com`). Firebase Hosting supports custom domains for free —
you only pay the registrar for the domain (~$10/yr).

---

## Migration steps

### 1. Register a domain
Any registrar works (Namecheap, Cloudflare Registrar, Porkbun, Google Domains).
Avoid a `.app` TLD unless you enable **HSTS** on the custom domain (it is
mandatory for `.app` and adds a setup step).

### 2. Connect it to Firebase Hosting
1. Firebase console → **Project settings** → **General** → **Add custom domain**.
2. Enter `yourdomain.com` (and `www.yourdomain.com` if you want both).
3. Firebase prints the DNS records — typically:
   - `A` record → `199.36.153.4`
   - `AAAA` record → `2001:4860:4802:32::a`
   - `CNAME` → `yourdomain.com` → `your-project.web.app`
4. Add the records at your registrar and wait for the certificate to be issued
   (automatic, usually minutes).

### 3. Update the app
Only **one line of code** depends on the domain — `src/util.rs`:

```rust
pub const SITE_URL: &str = "https://majehsemailviewer.web.app";  // -> your domain
```

This drives every public link (homepage privacy link, in-app privacy/terms
links). Then also update:
- `index.html` → `<link rel="canonical">`, `og:url`, and the static homepage's
  absolute policy links
- `privacy.html` / `terms.html` → `<link rel="canonical">`
- `index.html` → the `google-site-verification` meta tag (re-verify the new
  domain in Search Console)

### 4. Update Google Cloud OAuth
In **Google Cloud Console → APIs & Services → Credentials → your OAuth client**:
- **Authorized JavaScript origins**: `https://yourdomain.com`
- **Authorized redirect URIs**: `https://yourdomain.com/auth/callback/google`
  (keep the old `web.app` one only if you still serve the app there)

### 5. Update the OAuth consent screen
In **Google Cloud Console → OAuth consent screen → App information**:
- **Homepage**: `https://yourdomain.com`
- **Privacy policy**: `https://yourdomain.com/privacy.html`
  → this must be **identical** to the link on the app homepage
- **Application home page** / support email as well

### 6. Re-verify in Play Console
Add the new domain in Play Console → **App content → App access** and re-run the
homepage review.

---

## Notes
- The **Google Client Secret** is stored per-browser in `localStorage`. After the
  migration, each user must re-enter it once in **Settings → Google Client
  Secret** on the new domain (that is by design — the secret never ships in the
  app).
- `firebase.json` needs no change: Firebase serves the custom domain from the
  same `public` directory.
- Keep the `web.app` URL alive as a fallback, but make the **custom domain** the
  one registered with Google (OAuth + Play Console + consent screen).
