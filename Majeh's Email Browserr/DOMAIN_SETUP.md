# Owning Your Domain — required for OAuth / Play verification

## Current status of the three findings

| Finding | Cause | Status |
|---|---|---|
| "The website of your home page URL is not registered to you" | `majehsemailviewer.web.app` is a **shared Google domain** — the registrable domain (`web.app`) belongs to Google, so you cannot prove ownership | ❌ **Needs a custom domain** |
| "Your home page is behind a login page" | The reviewed build still had the **splash unlock form** | ✅ Fixed (splash is now a 900 ms brand intro; Home tab + static block are public) |
| "Your home page does not explain the purpose of your app" | The homepage had **no HTML text** — only WASM | ✅ Fixed (static `#static-home` block: 407 words, purpose + per-scope table + policy link) |

## Why waiting will not fix the ownership finding

Google's Trust & Safety team requires the homepage **and** the privacy policy to
live on *"a verified domain that you own"*, and explicitly warns against
third-party platforms where subdomain ownership cannot be verified
(`google sites`, `facebook`, `github.io`, and by the same rule `web.app`).

Developer-forum reports confirm that **verifying the `web.app`/hosting domain
in Search Console is not sufficient** — the team still asks for a domain under
your own ownership. Waiting 24 hours only helps *after* ownership of a
registrable domain has been verified. Retrying tomorrow with the same
`web.app` URL will be rejected again.

## What to do

### 1. Register a domain you own
Any registrar. Avoid the `.app` TLD unless you also enable **HSTS** on the
custom domain (mandatory for `.app`, one extra DNS step). `.com`/`.dev`/`.net`
are fine.

### 2. Point it at Firebase Hosting
Firebase console → **Project settings → General → Add custom domain** →
enter `yourdomain.com` (and `www` if you want both). Firebase prints the DNS
records — typically:
- `A` → `199.36.153.4`
- `AAAA` → `2001:4860:4802:32::a`
- `CNAME` → `yourdomain.com` → `your-project.web.app`

The TLS certificate is issued automatically (minutes, occasionally up to 24 h).

### 3. Update the code
```bash
./scripts/update_domain.sh yourdomain.com
```
This rewrites every hardcoded occurrence of the old domain in `index.html`,
`privacy.html`, `terms.html`, `robots.txt`, `sitemap.xml`, `src/util.rs` and
the docs, then prints the remaining manual steps. The single source of truth
is `util::SITE_URL` — everything else derives from it.

### 4. Update Google Cloud OAuth
**APIs & Services → Credentials → your OAuth client**
- **Authorized JavaScript origins**: `https://yourdomain.com`
- **Authorized redirect URIs**: `https://yourdomain.com/auth/callback/google`

### 5. Update the OAuth consent screen
**OAuth consent screen → App information**
- **Homepage**: `https://yourdomain.com`
- **Privacy policy**: `https://yourdomain.com/privacy.html`
  → must be **identical** to the link on the app homepage

### 6. Verify domain ownership (Search Console)

**What you have today is not enough.** The property in your screenshot —
`https://majehsemailviewer.web.app/` — is a **URL-prefix property**, verified
with the `google-site-verification` meta tag. That proves *"you can edit this
exact page"*. The Play/OAuth ownership check asks a different question: *"do
you own the registrable domain?"* — for that Google requires a **Domain
property** (not a URL-prefix property), proved with a **DNS TXT record**.

A `web.app` URL-prefix property can never be upgraded to a Domain property:
Domain properties are verified by adding a DNS TXT record, and you do not
control the DNS zone for `web.app` (Google does). That is precisely why the
ownership check fails.

**The correct flow with your own domain:**

1. Search Console → **Add property** → the **Domain** tab (not URL prefix).
2. Enter `yourdomain.com` with **no** `https://` and no `www.`.
3. Google shows a TXT record: `google-site-verification=XXXXXXXX`.
4. Add that TXT record at your **registrar** (Namecheap / Cloudflare /
   Porkbun) on the root domain (`@` host). Cloudflare: DNS → Records → Add →
   Type `TXT`, Name `@`, Content the token.
5. Click **Verify** — usually minutes, occasionally up to 48 h.
6. Use the **same Google account** that owns the GCP project and the Play
   developer account, so the verified domain is associated with your account.

Keep the existing `web.app` URL-prefix property as well — it is still valid for
`google-site-verification` if you keep serving the fallback, but the
**custom-domain** property is the one that satisfies the ownership check.

**Tip while you are in Search Console:** use **URL Inspection** on
`https://majehsemailviewer.web.app/` to see the *rendered* HTML Google
retrieves. It should show the static homepage block (brand, purpose, the
per-scope table, the privacy link) — that is the same content the Play
reviewer sees.

### 7. Resubmit
Two routes — either is valid:
- **Reply to the email** from Google's Third Party Data Safety Team. Their
  standard template asks you to reply with:
  > "Move your application homepage and privacy policy to domain under your
  > ownership / Update the homepage and privacy policy URLs in your Cloud
  > Console / Verify your domain ownership"
  and then confirm the homepage has been updated. Replying to the existing
  thread is usually **faster** than starting a new submission.
- **Or** update the homepage URL in the consent screen and press
  **Prepare for verification → Submit for verification**.

Then wait the 24 hours Google's notice mentions (it starts from the moment
ownership is verified) before retrying.

## Notes
- The **Google Client Secret** is stored per-browser in `localStorage`. After
  the migration each user must re-enter it once in **Settings → Google Client
  Secret** on the new domain — that is by design, the secret never ships in
  the app.
- `firebase.json` needs no change: Firebase serves the custom domain from the
  same `public` directory.
- Keep the `web.app` URL alive as a fallback, but make the **custom domain**
  the one registered with Google (OAuth, consent screen, Play Console).
