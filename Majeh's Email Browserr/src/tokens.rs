use leptos::*;
use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use crate::crypto::{encrypt, decrypt};

/// A single connected email account's OAuth tokens.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OAuthToken {
    pub provider: String,   // "gmail" | "outlook"
    pub email: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>, // unix ms
}

/// Serialized shape of the vault we write to localStorage / export file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultPayload {
    pub version: u32,
    pub tokens: Vec<OAuthToken>,
}

const VAULT_KEY: &str = "mejeh_email_vault";
const VAULT_VERSION: u32 = 1;

/// sessionStorage key holding OAuth tokens that arrived while the vault was
/// locked. They are merged into the vault the moment it is unlocked.
const PENDING_TOKENS_KEY: &str = "mejeh_pending_tokens";

/// Legacy single-token key written by older builds. Still read (and cleared)
/// so tokens captured by an older session are not lost.
const LEGACY_PENDING_TOKEN_KEY: &str = "pending_oauth_token";

// The verified vault password for this page load, shared by every component.
//
// NOTE: this MUST be a single module-level `thread_local!`. An earlier
// version declared the `thread_local!` *inside* `vault_password()` and
// *inside* `set_vault_password()`, which silently created two independent
// statics — the write went to one and the read came from the other, so
// `vault_password()` always returned `None` and every session-gated code
// path (pending-OAuth pickup, `persist_vault_now`) was dead.
thread_local! {
    static SESSION_PASSWORD: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// The verified vault password for this page load (`None` = locked).
pub fn vault_password() -> Option<String> {
    SESSION_PASSWORD.with(|p| p.borrow().clone())
}

/// Publish (or clear) the verified vault password for this page load.
pub fn set_vault_password(pw: Option<String>) {
    SESSION_PASSWORD.with(|p| *p.borrow_mut() = pw);
}

/// True once any successful unlock/create happened this page-load.
pub fn vault_is_unlocked() -> bool {
    vault_password().is_some()
}

/// Stash an OAuth token that could not be persisted yet because the vault
/// was locked. The token is merged into the vault the moment it is unlocked.
pub fn stash_pending_token(token: &OAuthToken) {
    let Some(storage) = session_storage() else {
        return;
    };
    let mut pending = read_pending_tokens(&storage);
    if !pending.iter().any(|t| t.email == token.email) {
        pending.push(token.clone());
    }
    if let Ok(json) = serde_json::to_string(&pending) {
        let _ = storage.set_item(PENDING_TOKENS_KEY, &json);
    }
    crate::util::log_info(&format!(
        "vault is locked — stashed the token for {} until it is unlocked",
        token.email
    ));
}

/// Take every stashed pending token and clear the stash.
///
/// Also reads the legacy single-token key (`pending_oauth_token`) so tokens
/// captured by an older build are not lost.
pub fn take_pending_tokens() -> Vec<OAuthToken> {
    let Some(storage) = session_storage() else {
        return Vec::new();
    };
    let mut pending = read_pending_tokens(&storage);
    if let Ok(Some(raw)) = storage.get_item(LEGACY_PENDING_TOKEN_KEY) {
        if !raw.is_empty() {
            if let Ok(token) = serde_json::from_str::<OAuthToken>(&raw) {
                if !pending.iter().any(|t| t.email == token.email) {
                    pending.push(token);
                }
            }
        }
        let _ = storage.remove_item(LEGACY_PENDING_TOKEN_KEY);
    }
    let _ = storage.remove_item(PENDING_TOKENS_KEY);
    pending
}

/// How many OAuth tokens are waiting for the vault to be unlocked.
pub fn pending_token_count() -> usize {
    let Some(storage) = session_storage() else {
        return 0;
    };
    let count = read_pending_tokens(&storage).len();
    let legacy = storage
        .get_item(LEGACY_PENDING_TOKEN_KEY)
        .ok()
        .flatten()
        .filter(|raw| !raw.is_empty())
        .is_some();
    count + usize::from(legacy)
}

/// Read the stashed pending tokens without clearing them.
fn read_pending_tokens(storage: &web_sys::Storage) -> Vec<OAuthToken> {
    storage
        .get_item(PENDING_TOKENS_KEY)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Vec<OAuthToken>>(&raw).ok())
        .unwrap_or_default()
}

/// sessionStorage of the current window, if available.
fn session_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok().flatten()
}

/// Encrypt `tokens` with `password` and write to localStorage.
/// Single choke point for all vault persistence.
pub fn save_vault(password: &str, tokens: &[OAuthToken]) -> Result<(), String> {
    let payload = VaultPayload {
        version: VAULT_VERSION,
        tokens: tokens.to_vec(),
    };
    let json =
        serde_json::to_string(&payload).map_err(|e| format!("serialize failed: {e}"))?;
    let encrypted = encrypt(&json, password)?;
    window_local_storage()
        .and_then(|ls| ls.set_item(VAULT_KEY, &encrypted).ok())
        .ok_or_else(|| "failed to write vault to storage".to_string())?;
    Ok(())
}

/// Re-encrypt the current vault contents with the session password and write
/// to localStorage. Called after any token refresh so a page reload restores
/// the *fresh* tokens instead of rolling back to the stale stored copy.
/// No-op when the vault is locked (no session password yet).
pub fn persist_vault_now(tokens: &[OAuthToken]) {
    let Some(pw) = vault_password() else {
        return;
    };
    if let Err(e) = save_vault(&pw, tokens) {
        leptos::logging::log!("vault auto-persist failed: {e}");
    }
}

/// Check whether an encrypted vault already exists in localStorage.
pub fn vault_exists() -> bool {
    window_local_storage()
        .and_then(|ls| ls.get_item(VAULT_KEY).ok().flatten())
        .is_some()
}

/// Decrypt the vault with `password` and return the stored tokens.
/// On success the password is published to the shared session.
pub fn unlock_vault(password: &str) -> Result<Vec<OAuthToken>, String> {
    let encrypted = window_local_storage()
        .and_then(|ls| ls.get_item(VAULT_KEY).ok().flatten())
        .ok_or_else(|| "No vault found".to_string())?;
    let json = decrypt(&encrypted, password)?;
    let payload: VaultPayload = serde_json::from_str(&json)
        .map_err(|e| format!("bad vault format: {e}"))?;
    set_vault_password(Some(password.to_string()));
    Ok(payload.tokens)
}

fn window_local_storage() -> Option<web_sys::Storage> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    Some(storage)
}

fn download_file(content: &str, filename: &str) {
    let window = web_sys::window().unwrap();
    let encoded = js_sys::encode_uri_component(content);
    let url = format!("data:application/json;charset=utf-8,{}", encoded);
    let a = window.document().unwrap().create_element("a").unwrap();
    a.set_attribute("href", &url).unwrap();
    a.set_attribute("download", filename).unwrap();
    a.set_attribute("style", "display:none").unwrap();
    window.document().unwrap().body().unwrap().append_child(&a).unwrap();
    a.unchecked_ref::<web_sys::HtmlAnchorElement>().click();
    let _ = window.document().unwrap().body().unwrap().remove_child(&a);
}

/// Reactive token store. Unlock with a password; tokens live in memory only.
#[component]
pub fn TokenVault(connected_accounts: RwSignal<Vec<OAuthToken>>) -> impl IntoView {
    let tokens = connected_accounts;
    let is_unlocked = create_rw_signal(false);
    let password_set = create_rw_signal(false);

    let existing = window_local_storage()
        .and_then(|ls| ls.get_item(VAULT_KEY).ok().flatten());
    password_set.set(existing.is_some());

    // Seed local state from the shared session: if Splash already unlocked the
    // vault this page-load, show unlocked immediately — no second prompt.
    // (`vault_is_unlocked()` is a plain thread-local read — no global signal,
    // so remounting this component on tab switches can't hit a disposed one.)
    if vault_is_unlocked() {
        is_unlocked.set(true);
    }

    let unlock = move |password: String| {
        let tokens = tokens;
        let is_unlocked = is_unlocked;
        move || {
            // `unlock_vault` decrypts the vault and publishes the password to
            // the shared session, so every component sees it as unlocked.
            match unlock_vault(&password) {
                Ok(mut decrypted) => {
                    // Merge any OAuth token that arrived while the vault was
                    // locked, then write the merged list back.
                    let pending = take_pending_tokens();
                    if !pending.is_empty() {
                        for token in pending {
                            let email = token.email.clone();
                            decrypted.retain(|t| t.email != email);
                            decrypted.push(token);
                        }
                        if let Err(e) = save_vault(&password, &decrypted) {
                            crate::util::log_error(&format!("vault persist failed: {e}"));
                        }
                    }
                    tokens.set(decrypted);
                    is_unlocked.set(true);
                    crate::util::log_info("vault unlocked");
                }
                Err(e) => {
                    crate::util::log_error(&format!("vault unlock failed: {e}"));
                    is_unlocked.set(false);
                }
            }
        }
    };

    let lock = move || {
        // Clear the session password FIRST so nothing tries to write the
        // vault again after the accounts were dropped.
        set_vault_password(None);
        tokens.set(Vec::new());
        is_unlocked.set(false);
        crate::util::log_info("vault locked");
        // NOTE: lock only clears in-memory state — the encrypted vault stays
        // in localStorage so the next unlock restores the accounts. (The old
        // code deleted the vault here, which is another way accounts "vanished".)
    };

    let save = move |new_tokens: Vec<OAuthToken>, password: String| {
        let tokens = tokens;
        let is_unlocked = is_unlocked;
        move || {
            match save_vault(&password, &new_tokens) {
                Ok(()) => {
                    // Publish the password first: the pending-OAuth effect
                    // reacts to `is_unlocked` and needs the session password
                    // to be there already.
                    set_vault_password(Some(password));
                    tokens.set(new_tokens);
                    is_unlocked.set(true);
                    password_set.set(true);
                }
                Err(e) => leptos::logging::log!("encrypt failed: {e}"),
            }
        }
    };

    let export = move |export_password: String| {
        let toks = tokens.get();
        move || {
            let payload = VaultPayload {
                version: VAULT_VERSION,
                tokens: toks,
            };
            let json = serde_json::to_string(&payload).unwrap();
            match encrypt(&json, &export_password) {
                Ok(encrypted) => {
                    let export_json = serde_json::to_string(&serde_json::json!({
                        "version": 1,
                        "kind": "mejeh-email-vault",
                        "data": encrypted
                    })).unwrap();
                    download_file(&export_json, "mejeh-email-vault.json");
                }
                Err(e) => leptos::logging::log!("export encrypt failed: {e}"),
            }
        }
    };

    let import = move |file_data: String, import_password: String| {
        let tokens = tokens.clone();
        let is_unlocked = is_unlocked.clone();
        move || {
            let parsed: serde_json::Value = match serde_json::from_str(&file_data) {
                Ok(v) => v,
                Err(e) => {
                    leptos::logging::log!("import parse failed: {e}");
                    return;
                }
            };
            let data = parsed
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or(&file_data);
            match decrypt(data, &import_password) {
                Ok(json) => {
                    let payload: VaultPayload = serde_json::from_str(&json)
                        .map_err(|e| format!("bad import format: {e}"))
                        .unwrap();
                    let payload2 = VaultPayload {
                        version: VAULT_VERSION,
                        tokens: payload.tokens.clone(),
                    };
                    let json2 = serde_json::to_string(&payload2).unwrap();
                    match encrypt(&json2, &import_password) {
                        Ok(encrypted) => {
                            if let Some(ls) = window_local_storage() {
                                let _ = ls.set_item(VAULT_KEY, &encrypted);
                            }
                            tokens.set(payload.tokens);
                            is_unlocked.set(true);
                            password_set.set(true);
                        }
                        Err(e) => leptos::logging::log!("import re-encrypt failed: {e}"),
                    }
                }
                Err(_) => leptos::logging::log!("import decrypt failed — wrong password?"),
            }
        }
    };

    view! {
        <div class="token-vault">
            <h3>"Connected Accounts"</h3>
            <Show when=move || !password_set.get()>
                <div class="vault-setup">
                    <p>"No vault password set yet. Create one to encrypt your accounts."</p>
                    <input type="password" placeholder="Create vault password" id="new-pw"/>
                    <button
                        on:click=move |_| {
                            let doc = web_sys::window().unwrap().document().unwrap();
                            let el = doc.get_element_by_id("new-pw").unwrap();
                            let pw: String = el.dyn_into::<web_sys::HtmlInputElement>().unwrap().value();
                            save(Vec::new(), pw)();
                        }
                    >
                        "Create Vault"
                    </button>
                </div>
            </Show>
            <Show when=move || password_set.get() && !is_unlocked.get()>
                <div class="vault-locked">
                    <p>"Vault is locked. Enter your vault password to load accounts."</p>
                    <input type="password" placeholder="Vault password" id="unlock-pw"/>
                    <button
                        on:click=move |_| {
                            let doc = web_sys::window().unwrap().document().unwrap();
                            let el = doc.get_element_by_id("unlock-pw").unwrap();
                            let pw: String = el.dyn_into::<web_sys::HtmlInputElement>().unwrap().value();
                            unlock(pw)();
                        }
                    >
                        "Unlock"
                    </button>
                </div>
            </Show>
            <Show when=move || is_unlocked.get()>
                <div class="vault-open">
                    <ul>
                        {move || {
                            tokens.get().into_iter().map(|t| {
                                view! {
                                    <li class="token-row">
                                        <span class="provider">{t.provider}</span>
                                        <span class="email">{t.email}</span>
                                    </li>
                                }
                            }).collect_view()
                        }}
                    </ul>
                    <div class="vault-actions">
                        <button
                            on:click=move |_| {
                                let doc = web_sys::window().unwrap().document().unwrap();
                                let el = doc.get_element_by_id("export-pw").unwrap();
                                let pw: String = el.dyn_into::<web_sys::HtmlInputElement>().unwrap().value();
                                export(pw)();
                            }
                        >
                            "Export Vault"
                        </button>
                        <input type="password" placeholder="Export password" id="export-pw"/>
                        <input type="file" id="import-file"/>
                        <input type="password" placeholder="Import password" id="import-pw"/>
                        <button
                            on:click=move |_| {
                                let doc = web_sys::window().unwrap().document().unwrap();
                                let input = doc.get_element_by_id("import-file").unwrap();
                                let file_input = input.unchecked_ref::<web_sys::HtmlInputElement>();
                                let files = js_sys::Reflect::get(file_input, &JsValue::from_str("files"))
                                    .ok()
                                    .filter(|v| !v.is_null() && !v.is_undefined())
                                    .and_then(|v| {
                                        let len = js_sys::Reflect::get(&v, &JsValue::from_str("length"))
                                            .ok()
                                            .and_then(|l| l.as_f64())
                                            .unwrap_or(0.0) as u32;
                                        if len > 0 {
                                            js_sys::Reflect::get(&v, &JsValue::from_str("0")).ok()
                                        } else {
                                            None
                                        }
                                    });
                                match files {
                                    Some(file) => {
                                        let file_blob = file.unchecked_into::<web_sys::Blob>();
                                        let reader = web_sys::FileReader::new().unwrap();
                                        let reader_clone = reader.clone();
                                        let onload = Closure::wrap(
                                            Box::new(move |_event: web_sys::Event| {
                                                let result = reader_clone.result().unwrap();
                                                if let Ok(text) = result.dyn_into::<js_sys::JsString>() {
                                                    let pw = doc.get_element_by_id("import-pw").unwrap()
                                                        .dyn_into::<web_sys::HtmlInputElement>().unwrap().value();
                                                    import(text.into(), pw)();
                                                }
                                            }) as Box<dyn FnMut(_)>
                                        );
                                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                        onload.forget();
                                        reader.read_as_text(&file_blob).unwrap();
                                    }
                                    None => {}
                                }
                            }
                        >
                            "Import Vault"
                        </button>
                        <button on:click=move |_| lock()>"Lock Vault"</button>
                    </div>
                </div>
            </Show>


        </div>
    }
}
