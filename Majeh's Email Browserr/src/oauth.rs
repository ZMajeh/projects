use serde::{Deserialize, Serialize};
use base64::Engine as _;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::tokens::OAuthToken;

/// sessionStorage key holding the CSRF `state` of the in-flight request.
pub const PKCE_STATE_KEY: &str = "oauth_pkce_state";
/// sessionStorage key holding the PKCE `code_verifier` of the in-flight request.
pub const PKCE_VERIFIER_KEY: &str = "oauth_pkce_verifier";
/// `type` of the `postMessage` the OAuth popup sends back to the opener.
pub const OAUTH_MESSAGE_TYPE: &str = "majeh-oauth-callback";

/// localStorage key the OAuth popup uses to hand the authorization code back
/// to the opener window.
///
/// The key is suffixed with the flow's `state`, so two tabs running the flow
/// at the same time cannot steal each other's result.
///
/// localStorage is shared by every same-origin tab/window and — unlike
/// `window.opener` — it is unaffected by `Cross-Origin-Opener-Policy`, which
/// Google may apply and which would otherwise sever the opener link. The
/// opener polls this key, so the hand-off works even in that case.
pub const OAUTH_RESULT_KEY: &str = "mejeh_oauth_result";

/// Prefix marking the `state` of a *popup* OAuth flow. The callback page
/// uses it to tell "I am the popup, hand the code back" apart from "I am the
/// main tab, handle the callback here".
pub const POPUP_STATE_PREFIX: &str = "p|";

/// Prefix marking the `state` of a *same-tab* (redirect) OAuth flow.
pub const TAB_STATE_PREFIX: &str = "t|";

/// The localStorage key a given flow's result is stored under.
pub fn oauth_result_key(state: &str) -> String {
    format!("{OAUTH_RESULT_KEY}_{state}")
}

/// Read the OAuth client ID, client secret, and canonical redirect URI from
/// the HTML page at runtime. Client secret is read from localStorage first
/// (so it never needs to be committed), falling back to the meta tag.
pub fn config() -> Result<(String, String, String), String> {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return Err("no window".to_string()),
    };
    let doc = match window.document() {
        Some(d) => d,
        None => return Err("no document".to_string()),
    };
    let client_id = doc
        .query_selector("#__oauth_client_id")
        .map_err(|e| format!("query_selector failed: {e:?}"))?
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        .and_then(|el| el.get_attribute("data-client-id"))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "missing #__oauth_client_id[data-client-id]".to_string())?;
    // Try localStorage first (never committed), then fall back to meta tag.
    let client_secret = window
        .local_storage()
        .ok()
        .flatten()
        .and_then(|ls| ls.get_item("oauth_client_secret").ok().flatten())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            doc.query_selector("#__oauth_client_id")
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                .and_then(|el| el.get_attribute("data-client-secret"))
        })
        .unwrap_or_default();
    let origin = window
        .location()
        .origin()
        .map_err(|e| format!("location.origin failed: {e:?}"))?;
    let redirect_uri = format!("{origin}/auth/callback/google");
    Ok((client_id, client_secret, redirect_uri))
}


/// OAuth provider identifier (Phase 1: Google only; Outlook behind the same shape).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OAuthProvider {
    Google,
    // Outlook, // Phase 2
}

impl OAuthProvider {
    pub fn auth_endpoint(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        }
    }

    pub fn token_endpoint(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://oauth2.googleapis.com/token",
        }
    }

    pub fn userinfo_endpoint(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://openidconnect.googleapis.com/v1/userinfo",
        }
    }

    pub fn scopes(&self) -> &'static str {
        match self {
            OAuthProvider::Google => {
                "openid email profile https://www.googleapis.com/auth/gmail.readonly"
            }
        }
    }

    /// Phase 2 (Outlook) — kept for the multi-provider vault layout.
    #[allow(dead_code)]
    pub fn vault_name(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "gmail",
        }
    }
}

/// Parse `code`, `state` and `error` out of the current URL's query string.
///
/// Google sends `error=access_denied` (instead of `code`) when the user
/// dismisses the consent screen, so the error is parsed as well.
pub fn callback_params() -> (Option<String>, Option<String>, Option<String>) {
    let Some(window) = web_sys::window() else {
        return (None, None, None);
    };
    let Ok(href) = window.location().href() else {
        return (None, None, None);
    };
    let Some(query) = href.split('?').nth(1) else {
        return (None, None, None);
    };
    let mut code = None;
    let mut state = None;
    let mut error = None;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let raw = parts.next().unwrap_or("");
        let value = urlencoding::decode(raw)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| raw.to_string());
        match key {
            "code" => code = Some(value),
            "state" => state = Some(value),
            "error" => error = Some(value),
            _ => {}
        }
    }
    (code, state, error)
}

/// True when this document is the OAuth **popup** that the app opened.
///
/// The `state` is generated by us and echoed back by Google, so the `p|`
/// prefix is a reliable marker that survives the round trip to Google (and
/// is not affected by `Cross-Origin-Opener-Policy` severing `window.opener`).
pub fn is_popup_callback() -> bool {
    let (_, state, _) = callback_params();
    state
        .as_deref()
        .map(|s| s.starts_with(POPUP_STATE_PREFIX))
        .unwrap_or(false)
}

/// The hand-off payload the popup leaves for the opener window.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PopupResult {
    pub state: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Hand the authorization code back to the app window that opened the popup.
///
/// The result is written to `localStorage`, which every same-origin window
/// shares. That keeps the hand-off working even when the browser severs
/// `window.opener` because of `Cross-Origin-Opener-Policy` (Google does set
/// COOP on its sign-in pages). A `postMessage` is sent as well so the opener
/// can react immediately when the opener link survived.
pub fn relay_code_to_opener() -> Result<(), String> {
    let (code, state, error) = callback_params();
    let Some(state) = state else {
        return Err("no state in the callback URL".into());
    };
    if !state.starts_with(POPUP_STATE_PREFIX) {
        return Err("this is not a popup callback".into());
    }
    let payload = serde_json::json!({
        "type": OAUTH_MESSAGE_TYPE,
        "state": state,
        "code": code,
        "error": error,
    });
    let json = payload.to_string();
    crate::util::log_info(&format!("popup handing result back to the opener (code: {})", code.is_some()));
    // Durable hand-off (survives a severed opener link). The key is scoped to
    // this flow's state, so another tab's flow cannot pick it up.
    if let Some(ls) = window_local_storage() {
        let _ = ls.set_item(&oauth_result_key(&state), &json);
    }
    // Fast path: notify the opener directly when the link is intact.
    if let Some(window) = web_sys::window() {
        if let Ok(opener) = window.opener() {
            if !opener.is_null() && !opener.is_undefined() {
                if let Ok(origin) = window.location().origin() {
                    let opener_window: web_sys::Window = opener.unchecked_into();
                    let _ = opener_window.post_message(&JsValue::from_str(&json), &origin);
                }
            }
        }
    }
    Ok(())
}

/// Read the popup hand-off result for `state` without removing it.
pub fn peek_popup_result(state: &str) -> Option<PopupResult> {
    let raw = window_local_storage()?
        .get_item(&oauth_result_key(state))
        .ok()
        .flatten()?;
    serde_json::from_str(&raw).ok()
}

/// Remove the popup hand-off result for `state` from storage.
pub fn clear_popup_result(state: &str) {
    if let Some(ls) = window_local_storage() {
        let _ = ls.remove_item(&oauth_result_key(state));
    }
}

fn window_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// Install a listener that stores a popup hand-off as soon as it arrives via
/// `postMessage` (the fast path). The durable hand-off is the localStorage
/// key, which the opener polls for — that also covers the case where the
/// browser severed `window.opener` because of `Cross-Origin-Opener-Policy`.
pub fn install_popup_listener() -> Result<(), String> {
    use wasm_bindgen::closure::Closure;
    let window = web_sys::window().ok_or("no window")?;
    let closure = Closure::wrap(Box::new(move |ev: web_sys::MessageEvent| {
        let Some(origin) = web_sys::window().and_then(|w| w.location().origin().ok()) else {
            return;
        };
        if ev.origin() != origin {
            return;
        }
        let Some(raw) = ev.data().as_string() else {
            return;
        };
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return;
        };
        if parsed.get("type").and_then(|t| t.as_str()) != Some(OAUTH_MESSAGE_TYPE) {
            return;
        }
        // Only accept a result for the request this tab actually started, so
        // a late message from an earlier flow can never be picked up.
        let Some(state) = parsed.get("state").and_then(|s| s.as_str()) else {
            return;
        };
        let stored_state = session_storage()
            .and_then(|s| s.get_item(PKCE_STATE_KEY).ok().flatten())
            .unwrap_or_default();
        if stored_state.is_empty() || state != stored_state {
            return;
        }
        if let Some(ls) = window_local_storage() {
            let _ = ls.set_item(&oauth_result_key(state), &raw);
        }
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);
    let target = web_sys::EventTarget::from(window);
    target
        .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
        .map_err(|e| format!("addEventListener failed: {e:?}"))?;
    // The listener must outlive this call - the app lives for the whole page.
    closure.forget();
    Ok(())
}

/// Open the Google consent screen in a popup window.
///
/// A popup keeps this app loaded (and the vault unlocked) while the user
/// signs in, so the new account can be saved immediately afterwards. The
/// popup only relays the authorization code back; the token exchange always
/// happens here, in the opener, where the PKCE verifier lives.
pub fn start_google_oauth_popup() -> Result<web_sys::Window, String> {
    let (client_id, client_secret, redirect_uri) = config()?;
    if client_secret.is_empty() {
        return Err("missing Google client secret — set it in Settings".into());
    }
    // The `p|` prefix marks this as a popup flow, so the callback page knows
    // it must hand the code back instead of handling it itself.
    let state = format!("{POPUP_STATE_PREFIX}{}", new_state()?);
    let verifier = new_code_verifier()?;
    let challenge = code_challenge(&verifier);
    if let Some(storage) = session_storage() {
        let _ = storage.set_item(PKCE_STATE_KEY, &state);
        let _ = storage.set_item(PKCE_VERIFIER_KEY, &verifier);
    }
    let auth_url = build_auth_url(&client_id, &redirect_uri, &state, &challenge);
    // Drop any result left over from an earlier sign-in attempt.
    clear_popup_result(&state);
    crate::util::log_info("opening the Google sign-in popup");
    let window = web_sys::window().ok_or("no window")?;
    window
        .open_with_url_and_target_and_features(
            &auth_url,
            "majeh_oauth_popup",
            "popup=yes,width=520,height=680,resizable=yes,scrollbars=yes",
        )
        .map_err(|e| format!("could not open the sign-in window: {e:?}"))?
        .ok_or_else(|| "the sign-in window was blocked".to_string())
}

/// Wait for the OAuth popup to hand back a result and exchange the code.
///
/// Polls the shared localStorage hand-off key, so it works even when the
/// browser severed the `window.opener` link (Google sends
/// `Cross-Origin-Opener-Policy` on its sign-in pages).
pub async fn await_popup_result(
    popup: &web_sys::Window,
    timeout_ms: i32,
) -> Result<OAuthToken, String> {
    let started = js_sys::Date::now();
    // The state is per-tab (sessionStorage), so another tab cannot claim
    // this flow's result.
    let state = session_storage()
        .and_then(|s| s.get_item(PKCE_STATE_KEY).ok().flatten())
        .unwrap_or_default();
    if state.is_empty() {
        return Err("no sign-in request is in progress in this tab".into());
    }
    loop {
        if let Some(result) = peek_popup_result(&state) {
            clear_popup_result(&state);
            if result.state != state {
                crate::util::log_error("popup result state mismatch — ignoring it");
                return Err("state mismatch".into());
            }
            if let Some(err) = result.error {
                return Err(format!("Google sign-in was not completed ({err})"));
            }
            let Some(code) = result.code else {
                return Err("no authorization code in the popup result".into());
            };
            return exchange_with_code(&code, &state).await;
        }
        // The user closed the sign-in window without finishing.
        if let Ok(true) = popup.closed() {
            return Err("the Google sign-in window was closed".into());
        }
        if js_sys::Date::now() - started > timeout_ms as f64 {
            return Err("timed out waiting for the Google sign-in window".into());
        }
        crate::util::sleep(300).await;
    }
}

/// sessionStorage of the current window, if available.
pub fn session_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok().flatten()
}

/// Exchange an authorization `code` for tokens, validating the CSRF `state`
/// against the value stored when the request was started.
pub async fn exchange_with_code(code: &str, state: &str) -> Result<OAuthToken, String> {
    let storage = session_storage();
    let stored_state = storage
        .as_ref()
        .and_then(|s| s.get_item(PKCE_STATE_KEY).ok().flatten())
        .unwrap_or_default();
    if state != stored_state {
        crate::util::log_error("state mismatch — the sign-in response did not match the request");
        return Err("state mismatch".into());
    }
    let verifier = storage
        .as_ref()
        .and_then(|s| s.get_item(PKCE_VERIFIER_KEY).ok().flatten())
        .unwrap_or_default();
    if let Some(s) = storage.as_ref() {
        let _ = s.remove_item(PKCE_STATE_KEY);
        let _ = s.remove_item(PKCE_VERIFIER_KEY);
    }
    let (client_id, client_secret, redirect_uri) = config()?;
    crate::util::log_info("exchanging the authorization code for tokens");
    let token_resp = exchange_code(&client_id, &client_secret, &redirect_uri, code, &verifier)
        .await
        .map_err(|e| format!("token exchange failed: {e}"))?;
    let user = fetch_userinfo(&token_resp.access_token)
        .await
        .map_err(|e| format!("userinfo fetch failed: {e}"))?;
    let token = extract_google_tokens(&token_resp, Some(user));
    crate::util::log_info(&format!("signed in as {}", token.email));
    Ok(token)
}

/// Response from the token endpoint (code exchange + refresh grant).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
}

/// Minimal Google userinfo payload (email is what we need for the account row).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub picture: Option<String>,
}

/// Build the Google consent URL. `state` binds the request (CSRF);
/// the `code_verifier` is stored in sessionStorage under `oauth_pkce_<state>`.
pub fn build_auth_url(client_id: &str, redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    let provider = OAuthProvider::Google;
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}&code_challenge={}&code_challenge_method=S256",
        provider.auth_endpoint(),
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(provider.scopes()),
        urlencoding::encode(state),
        urlencoding::encode(code_challenge),
    )
}

/// Generate a PKCE `code_verifier` (43–128 chars, URL-safe) from browser RNG.
pub fn new_code_verifier() -> Result<String, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let crypto = window.crypto().map_err(|_| "no crypto".to_string())?;
    let arr = js_sys::Uint8Array::new_with_length(64);
    crypto
        .get_random_values_with_array_buffer_view(&arr)
        .map_err(|e| format!("random failed: {e:?}"))?;
    let mut bytes = vec![0u8; 64];
    arr.copy_to(&mut bytes);
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes))
}

/// SHA-256 hash the verifier → base64url(no-pad) `code_challenge`.
pub fn code_challenge(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
}

/// Generate a short random `state` string for CSRF binding.
pub fn new_state() -> Result<String, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let crypto = window.crypto().map_err(|_| "no crypto".to_string())?;
    let arr = js_sys::Uint8Array::new_with_length(16);
    crypto
        .get_random_values_with_array_buffer_view(&arr)
        .map_err(|e| format!("random failed: {e:?}"))?;
    let mut bytes = vec![0u8; 16];
    arr.copy_to(&mut bytes);
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes))
}

fn form_body(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                urlencoding::encode(k),
                urlencoding::encode(v)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Exchange an authorization `code` + verifier for tokens (PKCE flow).
pub async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
    code_verifier: &str,
) -> Result<TokenResponse, String> {
    let body = form_body(&[
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ]);
    gloo_net::http::Request::post(OAuthProvider::Google.token_endpoint())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .map_err(|e| format!("request build failed: {e}"))?
        .send()
        .await
        .map_err(|e| format!("token request failed: {e}"))?
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("token parse failed (likely invalid_grant): {e}"))
}

/// Silent refresh: swap a `refresh_token` for a fresh access token.
/// Google requires the web-application `client_secret` here; it is the same
/// secret stored in Settings → "Google Client Secret" (localStorage).
pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenResponse, String> {
    if client_secret.is_empty() {
        return Err(
            "client_secret is missing — save it in Settings → Google Client Secret".into(),
        );
    }
    let body = form_body(&[
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
    ]);
    gloo_net::http::Request::post(OAuthProvider::Google.token_endpoint())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .map_err(|e| format!("request build failed: {e}"))?
        .send()
        .await
        .map_err(|e| format!("refresh request failed: {e}"))?
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("refresh failed (invalid_grant = revoked; reconnect needed): {e}"))
}

/// Fetch the user's profile (email) for the account row.
pub async fn fetch_userinfo(access_token: &str) -> Result<UserInfo, String> {
    let response = gloo_net::http::Request::get(OAuthProvider::Google.userinfo_endpoint())
        .header("Authorization", &format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("userinfo request failed: {e}"))?;
    response
        .json::<UserInfo>()
        .await
        .map_err(|e| format!("userinfo parse failed: {e}"))
}

/// Gmail API: list message IDs for the authenticated user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GmailListResponse {
    #[serde(default)]
    pub messages: Vec<GmailMessageRef>,
    #[serde(default)]
    pub result_size_estimate: u64,
    #[serde(default, rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GmailMessageRef {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}

/// Gmail API: get a single message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GmailMessage {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(default, rename = "labelIds")]
    pub label_ids: Vec<String>,
    #[serde(default)]
    pub snippet: String,
    #[serde(default, rename = "internalDate")]
    pub internal_date: Option<String>,
    #[serde(default)]
    pub payload: Option<GmailMessagePart>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GmailMessagePart {
    #[serde(default, rename = "mimeType")]
    pub mime_type: String,
    #[serde(default)]
    pub headers: Vec<GmailHeader>,
    #[serde(default)]
    pub body: Option<GmailBody>,
    #[serde(default)]
    pub parts: Vec<GmailMessagePart>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GmailHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GmailBody {
    #[serde(default)]
    pub data: Option<String>,
    #[serde(default)]
    pub size: u32,
}

/// Default number of messages fetched when the user doesn't specify a count.
pub const DEFAULT_MAX_RESULTS: u32 = 20;
/// Gmail's hard ceiling for `maxResults` on a single list call.
pub const MAX_RESULTS_LIMIT: u32 = 500;

/// List Gmail message IDs with an optional Gmail search query.
/// Always scoped to `labelIds=INBOX` so folder contents stay consistent.
/// `q` accepts Gmail search syntax, e.g. `after:<epoch_s> before:<epoch_s>`
/// for manual date-range fetches.
/// `max_results` caps how many message IDs are returned (clamped to Gmail's
/// 500-per-request limit).
pub async fn gmail_list_messages_query(
    access_token: &str,
    q: Option<&str>,
    max_results: u32,
) -> Result<GmailListResponse, String> {
    let max = max_results.clamp(1, MAX_RESULTS_LIMIT);
    let mut url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages?labelIds=INBOX&maxResults={max}"
    );
    if let Some(q) = q.filter(|s| !s.is_empty()) {
        url.push_str("&q=");
        url.push_str(&urlencoding::encode(q));
    }
    let response = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("gmail list request failed: {e}"))?;
    response
        .json::<GmailListResponse>()
        .await
        .map_err(|e| format!("gmail list parse failed: {e}"))
}

/// Decode base64url (Gmail body encoding, padding optional) into UTF-8 text.
pub fn decode_b64url(data: &str) -> Result<String, String> {
    let mut normalized = data.replace('-', "+").replace('_', "/");
    while normalized.len() % 4 != 0 {
        normalized.push('=');
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&normalized)
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("body utf-8 decode failed: {e}"))
}

/// Recursively collect `text/plain` leaf bodies from a message part tree,
/// skipping attachments, inline images, and HTML-only parts.
fn collect_plain_text(part: &GmailMessagePart, out: &mut Vec<String>) {
    if !part.parts.is_empty() {
        for child in &part.parts {
            collect_plain_text(child, out);
        }
        return;
    }
    let mime = part.mime_type.to_lowercase();
    let is_plain = mime.is_empty() || mime.starts_with("text/plain");
    if !is_plain {
        return;
    }
    if let Some(data) = part.body.as_ref().and_then(|b| b.data.as_ref()) {
        if let Ok(text) = decode_b64url(data) {
            if !text.trim().is_empty() {
                out.push(text);
            }
        }
    }
}

/// Extract the full plain-text body of a Gmail message (base64url decoded).
/// Falls back to the snippet when no text/plain part is present.
pub fn gmail_extract_body(msg: &GmailMessage) -> String {
    let mut parts = Vec::new();
    if let Some(payload) = msg.payload.as_ref() {
        collect_plain_text(payload, &mut parts);
    }
    if parts.is_empty() {
        return msg.snippet.clone();
    }
    parts.join("\n")
}

/// Parse an RFC 5322 address header value like `Jane Doe <jane@x.com>`
/// into a display name and bare address.
pub fn parse_address_header(value: &str) -> (String, String) {
    let trimmed = value.trim();
    if let Some(open) = trimmed.rfind('<') {
        if trimmed.ends_with('>') {
            let name = trimmed[..open].trim().trim_matches('"').to_string();
            let addr = trimmed[open + 1..trimmed.len() - 1].trim().to_string();
            if !addr.is_empty() {
                let display = if name.is_empty() { addr.clone() } else { name };
                return (display, addr);
            }
        }
    }
    (trimmed.to_string(), trimmed.to_string())
}

/// Fetch a single Gmail message by ID.
pub async fn gmail_get_message(access_token: &str, message_id: &str) -> Result<GmailMessage, String> {
    let url = format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{message_id}");
    let response = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("gmail get request failed: {e}"))?;
    response
        .json::<GmailMessage>()
        .await
        .map_err(|e| format!("gmail get parse failed: {e}"))
}

/// Handle the Google OAuth callback in the current tab (the non-popup path).
///
/// Reads `code` + `state` from the URL, verifies the state, and exchanges
/// the code for tokens.
pub async fn handle_google_callback() -> Result<(OAuthToken, UserInfo), String> {
    let (code, state, error) = callback_params();
    if let Some(err) = error {
        return Err(format!("Google sign-in was not completed ({err})"));
    }
    let (Some(code), Some(state)) = (code, state) else {
        return Err("no code in redirect".to_string());
    };
    let token = exchange_with_code(&code, &state).await?;
    // The user info is already part of the token; keep the old shape for the
    // callers that still expect it.
    let user = UserInfo {
        email: Some(token.email.clone()),
        name: None,
        picture: None,
    };
    Ok((token, user))
}

/// Extract a single OAuthToken from the response. If profile lookup is not
/// available, falls back to the email parsed out of the scope string.
pub fn extract_google_tokens(response: &TokenResponse, profile: Option<UserInfo>) -> OAuthToken {
    let access_token = response.access_token.clone();
    let email = profile
        .and_then(|p| p.email)
        .filter(|e| !e.is_empty())
        .or_else(|| {
            response
                .scope
                .as_ref()
                .and_then(|s| s.split(' ').find(|s| s.contains('@')).map(|s| s.to_string()))
        })
        .unwrap_or_default();
    let now_ms = js_sys::Date::now() as i64;
    OAuthToken {
        provider: "gmail".to_string(),
        email,
        access_token,
        refresh_token: response.refresh_token.clone(),
        expires_at: now_ms
            .checked_add(response.expires_in as i64 * 1000),
    }
}

/// Start the Google OAuth flow in the **current tab** (full-page redirect).
///
/// This is the fallback for when the browser blocks the sign-in popup. It
/// navigates the whole app away to Google and back, which reloads the app —
/// so the vault session is lost and the account is only saved once the vault
/// is unlocked again.
pub fn start_google_oauth() -> Result<(), String> {
    let (client_id, client_secret, redirect_uri) = config()?;
    if client_secret.is_empty() {
        return Err("missing Google client secret — set it in Settings".into());
    }
    // The `t|` prefix marks this as a same-tab flow, so the callback page
    // knows it must handle the code itself.
    let state = format!("{TAB_STATE_PREFIX}{}", new_state()?);
    let verifier = new_code_verifier()?;
    let challenge = code_challenge(&verifier);
    // Store state + verifier in sessionStorage so the callback can validate them.
    if let Some(storage) = session_storage() {
        let _ = storage.set_item(PKCE_STATE_KEY, &state);
        let _ = storage.set_item(PKCE_VERIFIER_KEY, &verifier);
    }
    // Build the consent URL and throw the user.
    let auth_url = build_auth_url(&client_id, &redirect_uri, &state, &challenge);
    crate::util::log_info("redirecting this tab to the Google sign-in page");
    web_sys::window()
        .ok_or_else(|| "no window".to_string())?
        .location()
        .set_href(&auth_url)
        .map_err(|e| format!("set_href failed: {e:?}"))?;
    Ok(())
}

