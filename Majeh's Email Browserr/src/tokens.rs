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

/// Check whether an encrypted vault already exists in localStorage.
pub fn vault_exists() -> bool {
    window_local_storage()
        .and_then(|ls| ls.get_item(VAULT_KEY).ok().flatten())
        .is_some()
}

/// Decrypt the vault with `password` and return the stored tokens.
pub fn unlock_vault(password: &str) -> Result<Vec<OAuthToken>, String> {
    let encrypted = window_local_storage()
        .and_then(|ls| ls.get_item(VAULT_KEY).ok().flatten())
        .ok_or_else(|| "No vault found".to_string())?;
    let json = decrypt(&encrypted, password)?;
    let payload: VaultPayload = serde_json::from_str(&json)
        .map_err(|e| format!("bad vault format: {e}"))?;
    Ok(payload.tokens)
}

/// Create a new vault with `password`, storing an empty token list.
/// Fails if a vault already exists.
pub fn create_vault(password: &str) -> Result<(), String> {
    if vault_exists() {
        return Err("vault already exists".to_string());
    }
    let payload = VaultPayload {
        version: VAULT_VERSION,
        tokens: Vec::new(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    let encrypted = encrypt(&json, password)?;
    window_local_storage()
        .and_then(|ls| ls.set_item(VAULT_KEY, &encrypted).ok())
        .ok_or_else(|| "failed to write vault to storage".to_string())?;
    Ok(())
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
    let vault_password = create_rw_signal(String::new());

    let existing = window_local_storage()
        .and_then(|ls| ls.get_item(VAULT_KEY).ok().flatten());
    password_set.set(existing.is_some());

    // When vault unlocks, check for a pending OAuth token from the callback flow
    // and add it to the vault automatically (encrypted with the vault password).
    let tokens_clone = tokens.clone();
    let is_unlocked_clone = is_unlocked.clone();
    let vault_password_clone = vault_password.clone();
    create_effect(move |_| {
        if is_unlocked_clone.get() {
            if let Ok(Some(storage)) = web_sys::window().unwrap().session_storage() {
                if let Ok(Some(pending)) = storage.get_item("pending_oauth_token") {
                    if !pending.is_empty() {
                        if let Ok(token) = serde_json::from_str::<OAuthToken>(&pending) {
                            let mut current = tokens_clone.get();
                            if !current.iter().any(|t| t.email == token.email) {
                                current.push(token.clone());
                                tokens_clone.set(current.clone());
                                // Persist to vault immediately
                                let pw = vault_password_clone.get();
                                if !pw.is_empty() {
                                    let payload = VaultPayload {
                                        version: VAULT_VERSION,
                                        tokens: current.clone(),
                                    };
                                    if let Ok(json) = serde_json::to_string(&payload) {
                                        if let Ok(encrypted) = crate::crypto::encrypt(&json, &pw) {
                                            if let Some(ls) = window_local_storage() {
                                                let _ = ls.set_item(VAULT_KEY, &encrypted);
                                            }
                                        }
                                    }
                                }
                                leptos::logging::log!(
                                    "OAuth token saved to vault for {}",
                                    token.email
                                );
                            }
                            let _ = storage.remove_item("pending_oauth_token");
                        }
                    }
                }
            }
        }
    });

    let unlock = move |password: String| {
        let tokens = tokens.clone();
        let is_unlocked = is_unlocked.clone();
        let vault_password = vault_password.clone();
        move || {
            match window_local_storage()
                .and_then(|ls| ls.get_item(VAULT_KEY).ok().flatten())
            {
                Some(encrypted) => match decrypt(&encrypted, &password) {
                    Ok(json) => {
                        let payload: VaultPayload = serde_json::from_str(&json)
                            .map_err(|e| format!("bad vault format: {e}"))
                            .unwrap();
                        tokens.set(payload.tokens);
                        is_unlocked.set(true);
                        vault_password.set(password);
                    }
                    Err(_) => is_unlocked.set(false),
                },
                None => is_unlocked.set(false),
            }
        }
    };

    let lock = move || {
        tokens.set(Vec::new());
        is_unlocked.set(false);
        vault_password.set(String::new());
        if let Some(ls) = window_local_storage() {
            let _ = ls.remove_item(VAULT_KEY);
        }
    };

    let save = move |new_tokens: Vec<OAuthToken>, password: String| {
        let tokens = tokens.clone();
        let is_unlocked = is_unlocked.clone();
        move || {
            let payload = VaultPayload {
                version: VAULT_VERSION,
                tokens: new_tokens.clone(),
            };
            let json = serde_json::to_string(&payload).unwrap();
            match encrypt(&json, &password) {
                Ok(encrypted) => {
                    if let Some(ls) = window_local_storage() {
                        let _ = ls.set_item(VAULT_KEY, &encrypted);
                    }
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
