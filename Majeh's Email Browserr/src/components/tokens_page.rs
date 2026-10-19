use leptos::*;
use wasm_bindgen::JsCast;
use crate::components::icons::Icon;
use crate::data::{save_email_cache, Email};
use crate::oauth::{config, refresh_access_token};
use crate::tokens::OAuthToken;
use std::collections::HashMap;

fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}

/// Rough humanized duration from milliseconds: "45s", "12m", "3h 20m", "2d".
fn humanize(ms: i64) -> String {
    let s = (ms / 1000).max(0);
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else if s < 86_400 {
        format!("{}h {}m", s / 3600, (s % 3600) / 60)
    } else {
        format!("{}d", s / 86_400)
    }
}

/// (css kind, display text) for a token's expiry state.
fn expiry_state(t: &OAuthToken) -> (&'static str, String) {
    match t.expires_at {
        None => ("unknown", "No expiry info".to_string()),
        Some(exp) => {
            let delta = exp - now_ms();
            if delta > 0 {
                ("active", format!("Active — expires in {}", humanize(delta)))
            } else {
                ("expired", format!("Expired {} ago", humanize(-delta)))
            }
        }
    }
}

/// Apply a successful refresh response to one account in the shared list.
fn apply_refresh(list: &mut Vec<OAuthToken>, email: &str, resp: &crate::oauth::TokenResponse) {
    if let Some(t) = list.iter_mut().find(|t| t.email == email) {
        t.access_token = resp.access_token.clone();
        if let Some(new_rt) = resp.refresh_token.clone() {
            t.refresh_token = Some(new_rt);
        }
        t.expires_at = Some(now_ms() + resp.expires_in as i64 * 1000);
    }
}

/// Tokens page: one row per connected account showing its email, access and
/// refresh tokens, live active/expired state, a per-row refresh button, and a
/// "Refresh all expired" action on top.
#[component]
pub fn TokensPage(
    connected_accounts: RwSignal<Vec<OAuthToken>>,
    email_cache: RwSignal<HashMap<String, Vec<Email>>>,
) -> impl IntoView {
    // Emails currently being refreshed (row spinners).
    let refreshing = create_rw_signal(Vec::<String>::new());
    // Status/error line for the last action.
    let status = create_rw_signal(Option::<String>::None);

    // Force-refresh a single account's access token.
    let refresh_one = move |email: String| {
        if refreshing.get().contains(&email) {
            return;
        }
        let token = match connected_accounts.get().into_iter().find(|t| t.email == email) {
            Some(t) => t,
            None => return,
        };
        let refresh_token = match token.refresh_token.clone().filter(|s| !s.is_empty()) {
            Some(rt) => rt,
            None => {
                status.set(Some(format!(
                    "{email}: no refresh token — reconnect the account in Settings."
                )));
                return;
            }
        };
        refreshing.update(|r| r.push(email.clone()));
        wasm_bindgen_futures::spawn_local(async move {
            let result = match config() {
                Ok((client_id, client_secret, _)) => {
                    refresh_access_token(&client_id, &client_secret, &refresh_token).await
                }
                Err(e) => Err(format!("OAuth config missing: {e}")),
            };
            match result {
                Ok(resp) => {
                    connected_accounts.update(|list| apply_refresh(list, &email, &resp));
                    status.set(Some(format!("{email}: token refreshed ✓")));
                }
                Err(e) => status.set(Some(format!("{email}: refresh failed — {e}"))),
            }
            refreshing.update(|r| r.retain(|x| x != &email));
        });
    };


    // Refresh every account whose token is currently expired (sequentially).
    let refresh_all_expired = move || {
        let with_refresh: Vec<OAuthToken> = connected_accounts
            .get()
            .into_iter()
            .filter(|t| matches!(t.expires_at, Some(exp) if exp <= now_ms()))
            .filter(|t| t.refresh_token.as_deref().is_some_and(|s| !s.is_empty()))
            .collect();
        if with_refresh.is_empty() {
            status.set(Some(
                "Nothing to refresh — no expired tokens with a refresh token.".into(),
            ));
            return;
        }
        let total = with_refresh.len();
        wasm_bindgen_futures::spawn_local(async move {
            let mut ok_count = 0;
            for (i, token) in with_refresh.into_iter().enumerate() {
                refreshing.update(|r| {
                    if !r.contains(&token.email) {
                        r.push(token.email.clone());
                    }
                });
                status.set(Some(format!(
                    "Refreshing expired tokens — {}/{} ({})…",
                    i + 1,
                    total,
                    token.email
                )));
                let outcome = match config() {
                    Ok((client_id, client_secret, _)) => {
                        refresh_access_token(
                            &client_id,
                            &client_secret,
                            token.refresh_token.as_deref().unwrap_or_default(),
                        )
                        .await
                    }
                    Err(e) => Err(format!("OAuth config missing: {e}")),
                };
                match outcome {
                    Ok(resp) => {
                        connected_accounts.update(|list| {
                            apply_refresh(list, &token.email, &resp)
                        });
                        ok_count += 1;
                    }
                    Err(e) => status.set(Some(format!("{} failed: {e}", token.email))),
                }
                refreshing.update(|r| r.retain(|x| x != &token.email));
            }
            status.set(Some(format!(
                "Done — refreshed {ok_count} of {total} expired token(s)."
            )));
        });
    };


    view! {
        <section class="tokens-page">
            <div class="tokens-header">
                <div>
                    <h2>"Tokens"</h2>
                    <p class="tokens-sub">
                        {move || {
                            let list = connected_accounts.get();
                            let total = list.len();
                            let (mut active, mut expired, mut unknown) = (0usize, 0usize, 0usize);
                            for t in &list {
                                match expiry_state(t).0 {
                                    "active" => active += 1,
                                    "expired" => expired += 1,
                                    _ => unknown += 1,
                                }
                            }
                            format!(
                                "{total} connected account(s) — {active} active, {expired} expired, {unknown} unknown"
                            )
                        }}
                    </p>
                </div>
                <button class="btn-refresh-all" on:click=move |_| refresh_all_expired()>
                    <Icon name="refresh"/>
                    "Refresh all expired"
                </button>
                <button class="btn-download" on:click=move |_| download_emails(email_cache.get())>
                    <Icon name="download"/>
                    "Download Emails Backup"
                </button>
                <label class="btn-upload" style="display:flex;align-items:center;gap:6px;">
                    <Icon name="file"/>
                    "Upload Emails Backup"
                    <input type="file" accept=".json" style="display:none" on:change=move |ev| {
            let file_input: web_sys::HtmlInputElement = event_target(&ev);
            if let Some(files) = file_input.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let cache = email_cache;
                    let onload = wasm_bindgen::closure::Closure::wrap(Box::new(
                        move |_event: web_sys::Event| {
                            if let Ok(result) = reader_clone.result() {
                                if let Some(text) = result.as_string() {
                                    match serde_json::from_str::<HashMap<String, Vec<Email>>>(&text) {
                                        Ok(imported) => {
                                            cache.update(|c| {
                                                for (email, emails) in imported {
                                                    c.entry(email).or_insert_with(Vec::new).extend(emails);
                                                }
                                            });
                                            save_email_cache(&cache.get_untracked());
                                            status.set(Some("Emails backup imported ✓".to_string()));
                                        }
                                        Err(_) => {
                                            status.set(Some("Invalid backup file".to_string()));
                                        }
                                    }
                                }
                            }
                        },
                    )
                        as Box<dyn FnMut(web_sys::Event)>);
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    let _ = reader.read_as_text(&file.unchecked_ref::<web_sys::Blob>());
                }
            }
                    }/>
                </label>
            </div>
            <p class="tokens-note">
                "Tokens are sensitive — avoid sharing screenshots. Refreshes update this session; the encrypted vault is rewritten from Settings."
            </p>
            {move || status.get().map(|s| view! { <div class="tokens-status">{s}</div> })}
            {move || {
                let list = connected_accounts.get();
                if list.is_empty() {
                    return view! {
                        <div class="tokens-empty">
                            <Icon name="shield" class="placeholder-icon"/>
                            <p>"No connected accounts yet — connect one in Settings to see its tokens here."</p>
                        </div>
                    }
                    .into_view();
                }
                view! {
                    <div class="token-table">
                        <div class="token-row token-row-head">
                            <span>"Account"</span>
                            <span>"Access token"</span>
                            <span>"Refresh token"</span>
                            <span>"Status"</span>
                            <span>"Action"</span>
                        </div>

                        {list
                            .into_iter()
                            .map(|t| {
                                let email = t.email.clone();
                                let provider = t.provider.clone();
                                let access = t.access_token.clone();
                                let refresh = t.refresh_token.clone().unwrap_or_default();
                                let (kind, state_text) = expiry_state(&t);
                                let busy = refreshing.get().contains(&email);
                                let no_refresh = refresh.is_empty();
                                let email_for_click = email.clone();
                                let access_for_copy = access.clone();
                                let refresh_for_copy = refresh.clone();
                                view! {
                                    <div class="token-row" class:expired=kind == "expired">
                                        <div class="token-cell">
                                            <span class="token-email">{email.clone()}</span>
                                            <span class="token-provider">{provider}</span>
                                        </div>
                                        <div class="token-cell">
                                            <span class="token-label">"Access token"</span>
                                            <code>{access}</code>
                                            <button
                                                class="btn-copy"
                                                on:click=move |_| write_clipboard(&access_for_copy)
                                            >
                                                "Copy"
                                            </button>
                                        </div>
                                        <div class="token-cell">
                                            <span class="token-label">"Refresh token"</span>
                                            {if no_refresh {
                                                view! { <span class="token-none">"— none —"</span> }
                                                    .into_view()
                                            } else {
                                                view! { <code>{refresh}</code> }.into_view()
                                            }}
                                            {if !no_refresh {
                                                Some(view! {
                                                    <button
                                                        class="btn-copy"
                                                        on:click=move |_| write_clipboard(&refresh_for_copy)
                                                    >
                                                        "Copy"
                                                    </button>
                                                })
                                            } else {
                                                None
                                            }}
                                        </div>
                                        <div class="token-cell">
                                            <span
                                                class="token-state"
                                                class:active=kind == "active"
                                                class:expired=kind == "expired"
                                                class:unknown=kind == "unknown"
                                            >
                                                {state_text}
                                            </span>
                                        </div>
                                        <div class="token-cell">
                                            <button
                                                class="btn-refresh"
                                                disabled=busy || no_refresh
                                                on:click=move |_| refresh_one(email_for_click.clone())
                                            >
                                                <Icon name="refresh" class=if busy { "spin" } else { "" }/>
                                                {if busy { "Refreshing…" } else { "Refresh" }}
                                            </button>
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                }
                .into_view()
            }}
        </section>
    }
}

/// Best-effort clipboard write (no error UI — copies are convenience only).
fn write_clipboard(text: &str) {
    if let Some(win) = web_sys::window() {
        let _ = win.navigator().clipboard().write_text(text);
    }
}

/// Serialize the email cache to JSON and trigger a browser download.
fn download_emails(cache: HashMap<String, Vec<Email>>) {
    let Ok(json) = serde_json::to_string_pretty(&cache) else {
        return;
    };
    let window = web_sys::window().unwrap();
    let encoded = js_sys::encode_uri_component(&json);
    let url = format!("data:application/json;charset=utf-8,{encoded}");
    let a = window.document().unwrap().create_element("a").unwrap();
    a.set_attribute("href", &url).unwrap();
    a.set_attribute("download", "majeh-emails-backup.json").unwrap();
    a.set_attribute("style", "display:none").unwrap();
    window.document().unwrap().body().unwrap().append_child(&a).unwrap();
    a.unchecked_ref::<web_sys::HtmlAnchorElement>().click();
    let _ = window.document().unwrap().body().unwrap().remove_child(&a);
}

