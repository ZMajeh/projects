use serde::{Deserialize, Serialize};
use base64::Engine as _;
use wasm_bindgen::JsCast;

use crate::tokens::OAuthToken;

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

    pub fn vault_name(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "gmail",
        }
    }
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
pub async fn refresh_access_token(
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, String> {
    let body = form_body(&[
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
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
    #[serde(default)]
    pub payload: Option<GmailMessagePart>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GmailMessagePart {
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

/// List Gmail message IDs (max 20 by default).
pub async fn gmail_list_messages(access_token: &str) -> Result<GmailListResponse, String> {
    let url = "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults=20";
    let response = gloo_net::http::Request::get(url)
        .header("Authorization", &format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("gmail list request failed: {e}"))?;
    response
        .json::<GmailListResponse>()
        .await
        .map_err(|e| format!("gmail list parse failed: {e}"))
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

/// Handle the Google OAuth callback. Reads the `code` from the URL query string,
/// verifies the `state`, exchanges the code for tokens, and returns them.
pub async fn handle_google_callback() -> Result<(OAuthToken, UserInfo), String> {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return Err("no window".to_string()),
    };
    let url = window.location().href().unwrap_or_default();
    // Parse ?code=...&state=...
    let params = url
        .split('?')
        .nth(1)
        .unwrap_or("")
        .split('&')
        .map(|pair| {
            let mut it = pair.splitn(2, '=');
            (it.next().unwrap_or("").to_string(), it.next().unwrap_or("").to_string())
        })
        .collect::<std::collections::HashMap<String, String>>();

    let code = params
        .get("code")
        .cloned()
        .ok_or_else(|| "no code in redirect".to_string())?;
    let state = params.get("state").cloned().unwrap_or_default();
    // Verify state against sessionStorage to mitigate CSRF.
    let stored_state = window
        .session_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item("oauth_pkce_state").ok())
        .flatten()
        .unwrap_or_default();
    if state != stored_state {
        return Err("state mismatch".to_string());
    }
    // Read the verifier BEFORE clearing sessionStorage entries.
    let verifier = window
        .session_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item("oauth_pkce_verifier").ok())
        .flatten()
        .unwrap_or_default();
    // Clean up sessionStorage now that we have what we need.
    if let Ok(Some(storage)) = window.session_storage() {
        let _ = storage.remove_item("oauth_pkce_state");
        let _ = storage.remove_item("oauth_pkce_verifier");
    }
    // Build the redirect URI the same way we did in start_google_oauth.
    let (client_id, client_secret, redirect_uri) = config().map_err(|e| e.to_string())?;
    let token_resp = exchange_code(&client_id, &client_secret, &redirect_uri, &code, &verifier)
        .await
        .map_err(|e| format!("token exchange failed: {e}"))?;
    let user = fetch_userinfo(&token_resp.access_token)
        .await
        .map_err(|e| format!("userinfo fetch failed: {e}"))?;
    let token = extract_google_tokens(&token_resp, Some(user.clone()));
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

pub fn start_google_oauth() -> Result<(), String> {
    let (client_id, _client_secret, redirect_uri) = config()?;

    let state = new_state()?;
    let verifier = new_code_verifier()?;
    let challenge = code_challenge(&verifier);
    // Store state + verifier in sessionStorage so the callback can validate them.
    if let Ok(Some(storage)) = web_sys::window().unwrap().session_storage() {
        let _ = storage.set_item("oauth_pkce_state", &state);
        let _ = storage.set_item("oauth_pkce_verifier", &verifier);
    }
    // Build the consent URL and throw the user.
    let auth_url = build_auth_url(&client_id, &redirect_uri, &state, &challenge);
    web_sys::window()
        .ok_or_else(|| "no window".to_string())?
        .location()
        .set_href(&auth_url)
        .map_err(|e| format!("set_href failed: {e:?}"))?;
    Ok(())
}

