use leptos::*;
use serde_json;
use std::collections::HashMap;
use crate::components::{
    about::About,
    email_list::{EmailList, ListFilter},
    fetch_panel::FetchPanel,
    navigation::{BottomNavigation, Tab},
    reading_pane::ReadingPane,
    settings::Settings,
    sidebar::Sidebar,
    splash::Splash,
    tokens_page::TokensPage,
    top_bar::TopBar,
};
use crate::util::sleep;
use crate::data::{emails, format_email_time, next_email_id, Email, FetchTask, Folder, TaskStatus};
use crate::oauth::{
    config, gmail_extract_body, gmail_get_message, gmail_list_messages_query,
    parse_address_header, refresh_access_token,
};
use crate::tokens::OAuthToken;

/// Convert a fetched Gmail message into a display `Email`.
fn gmail_msg_to_email(msg: &crate::oauth::GmailMessage) -> Email {
    let header = |name: &str| -> String {
        msg.payload
            .as_ref()
            .and_then(|p| p.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)))
            .map(|h| h.value.clone())
            .unwrap_or_default()
    };
    let subject = {
        let s = header("Subject");
        if s.is_empty() { "(No Subject)".into() } else { s }
    };
    let (sender, address) = {
        let from = header("From");
        if from.is_empty() {
            ("Unknown".to_string(), String::new())
        } else {
            parse_address_header(&from)
        }
    };
    let to_field = header("To");
    let timestamp_ms = msg
        .internal_date
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let time = if timestamp_ms > 0 {
        format_email_time(timestamp_ms)
    } else {
        String::new()
    };
    let preview = msg.snippet.clone();
    // Full plain-text body (base64url decoded from payload parts), snippet fallback.
    let full = gmail_extract_body(msg);
    let body: Vec<String> = full
        .split('\n')
        .map(|l| l.trim().trim_end_matches('\r').trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let body = if body.is_empty() {
        vec![preview.clone()]
    } else {
        body
    };
    let initial = sender
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .next()
        .unwrap_or('?')
        .to_string();
    let unread = msg.label_ids.iter().any(|l| l == "UNREAD");
    let starred = msg.label_ids.iter().any(|l| l == "STARRED");
    let folder = if msg.label_ids.iter().any(|l| l == "SENT") {
        Folder::Sent
    } else if msg.label_ids.iter().any(|l| l == "DRAFT") {
        Folder::Drafts
    } else if msg.label_ids.iter().any(|l| l == "TRASH") {
        Folder::Trash
    } else {
        Folder::Inbox
    };
    Email {
        id: next_email_id(),
        msg_id: msg.id.clone(),
        sender,
        address,
        to_field,
        subject,
        preview,
        time,
        timestamp_ms,
        initial,
        color: "#4285f4".into(),
        unread,
        starred,
        folder,
        body,
    }
}

/// Parse a `<input type="date">` value ("YYYY-MM-DD") into local-midnight epoch ms.
fn date_input_to_ms(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // A date-time without a timezone is interpreted as local time by JS `Date`.
    let iso = format!("{year:04}-{month:02}-{day:02}T00:00:00");
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(&iso));
    let ms = date.get_time();
    if ms.is_nan() {
        None
    } else {
        Some(ms as i64)
    }
}

/// Refresh the access token if it expires within 60s. Refresh happens at the
/// session level; the encrypted vault is only rewritten by TokenVault, which
/// is the component that holds the user's password.
async fn ensure_fresh_token(token: &mut OAuthToken) {
    let expires_at = match token.expires_at {
        Some(t) => t,
        None => return,
    };
    if (js_sys::Date::now() as i64) + 60_000 < expires_at {
        return;
    }
    let refresh_token = match token.refresh_token.clone() {
        Some(t) if !t.is_empty() => t,
        _ => {
            leptos::logging::log!(
                "access token for {} expired and no refresh token available",
                token.email
            );
            return;
        }
    };
    let (client_id, client_secret, _) = match config() {
        Ok(c) => c,
        Err(e) => {
            leptos::logging::log!("oauth config missing for token refresh: {e}");
            return;
        }
    };
    match refresh_access_token(&client_id, &client_secret, &refresh_token).await {
        Ok(resp) => {
            token.access_token = resp.access_token;
            if let Some(rt) = resp.refresh_token {
                token.refresh_token = Some(rt);
            }
            token.expires_at = Some((js_sys::Date::now() as i64) + resp.expires_in as i64 * 1000);
            leptos::logging::log!("access token refreshed for {}", token.email);
        }
        Err(e) => {
            // Keep going with the old token; the API call surfaces a real
            // error if it is truly dead.
            leptos::logging::log!("token refresh failed for {}: {e}", token.email);
        }
    }
}

/// Root application component: top bar, sidebar, content panes, bottom nav (mobile).
#[component]
pub fn App() -> impl IntoView {
    let emails_store = create_rw_signal(emails());
    let selected_id = create_rw_signal::<Option<usize>>(None);
    let active_folder = create_rw_signal(Folder::Inbox);
    let list_filter = create_rw_signal(ListFilter::All);
    let query = create_rw_signal(String::new());
    let (drawer_open, set_drawer_open) = create_signal(false);
    // Shared list of connected OAuth accounts (from the encrypted vault).
    let connected_accounts = create_rw_signal(Vec::<OAuthToken>::new());
    // The currently active OAuth account (when user clicks one in sidebar).
    let active_account = create_rw_signal::<Option<OAuthToken>>(None);
    // Fetched emails cached per account email — re-clicking shows them instantly.
    let email_cache = create_rw_signal(HashMap::<String, Vec<Email>>::new());
    // In-flight/finished fetch jobs, each with its own progress bar.
    let fetch_tasks = create_rw_signal(Vec::<FetchTask>::new());
    // Last user-facing fetch/OAuth error (dismissed by the user).
    let fetch_error = create_rw_signal(Option::<String>::None);
    // Splash screen: shown for 3 seconds on launch, then dismissed.
    let (splash_done, set_splash_done) = create_signal(false);
    wasm_bindgen_futures::spawn_local(async move {
        sleep(3000).await;
        set_splash_done.set(true);
    });

    // Selecting an email marks it read and (on mobile) opens the reading pane.
    let select_email = move |id: usize| {
        emails_store.update(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.id == id) {
                e.unread = false;
            }
        });
        selected_id.set(Some(id));
    };

    // Keep star/read toggles in the per-account cache so they survive
    // account switches. Guarded so cross-account writes can't happen, and
    // fully untracked so the cache write cannot retrigger this effect.
    create_effect(move |_| {
        let list = emails_store.get();
        let acc = active_account.get();
        if list.iter().any(|e| !e.msg_id.is_empty()) {
            if let Some(acc) = acc {
                untrack(move || {
                    email_cache.update(|cache| {
                        if let Some(entry) = cache.get_mut(&acc.email) {
                            let belongs = list
                                .iter()
                                .any(|e| entry.iter().any(|x| x.msg_id == e.msg_id));
                            if belongs {
                                *entry = list;
                            }
                        }
                    });
                });
            }
        }
    });


    // Fetch emails for an account via the Gmail API, with a per-task progress
    // bar. `range = Some((from, to))` applies a manual date filter (Gmail
    // `after:`/`before:` query); `None` fetches the latest inbox messages.
    let spawn_fetch = move |token: OAuthToken, range: Option<(String, String)>| {
        let from = range.as_ref().map(|r| r.0.clone()).unwrap_or_default();
        let to = range.as_ref().map(|r| r.1.clone()).unwrap_or_default();
        let label = if from.is_empty() && to.is_empty() {
            String::new()
        } else {
            format!(
                "{} → {}",
                if from.is_empty() { "…" } else { &from },
                if to.is_empty() { "…" } else { &to }
            )
        };
        let key = format!("{}|{}|{}", token.email, from, to);
        // Same account+range already fetching — don't start a duplicate.
        if fetch_tasks
            .get()
            .iter()
            .any(|t| t.key == key && t.status == TaskStatus::Running)
        {
            return;
        }
        fetch_error.set(None);
        // Drop a stale finished row for this task, then add the fresh one.
        fetch_tasks.update(|ts| {
            ts.retain(|t| t.key != key || t.status != TaskStatus::Done);
            ts.push(FetchTask {
                key: key.clone(),
                account: token.email.clone(),
                label,
                total: 0,
                fetched: 0,
                status: TaskStatus::Running,
            });
        });

        wasm_bindgen_futures::spawn_local(async move {
            let mut token = token;
            ensure_fresh_token(&mut token).await;
            // Reflect the refreshed token back into the sidebar's account list
            // and the active account so later calls reuse it.
            connected_accounts.update(|list| {
                if let Some(t) = list.iter_mut().find(|t| t.email == token.email) {
                    t.access_token = token.access_token.clone();
                    t.expires_at = token.expires_at;
                    if token.refresh_token.is_some() {
                        t.refresh_token = token.refresh_token.clone();
                    }
                }
            });
            active_account.update(|acc| {
                if let Some(t) = acc.as_mut() {
                    if t.email == token.email {
                        t.access_token = token.access_token.clone();
                        t.expires_at = token.expires_at;
                        if token.refresh_token.is_some() {
                            t.refresh_token = token.refresh_token.clone();
                        }
                    }
                }
            });

            // Optional date-range query: `after:`/`before:` use epoch seconds;
            // `before` is exclusive, so add a day to include the whole "to" day.
            let q = if from.is_empty() && to.is_empty() {
                None
            } else {
                let mut parts = Vec::new();
                if let Some(ms) = date_input_to_ms(&from) {
                    parts.push(format!("after:{}", ms / 1000));
                }
                if let Some(ms) = date_input_to_ms(&to) {
                    parts.push(format!("before:{}", (ms + 86_400_000) / 1000));
                }
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(" "))
                }
            };


            // Small helper to patch this task's row in the shared signal.
            let set_task = |f: Box<dyn FnOnce(&mut FetchTask)>| {
                fetch_tasks.update(|ts| {
                    if let Some(t) = ts.iter_mut().find(|t| t.key == key) {
                        f(t);
                    }
                });
            };

            match gmail_list_messages_query(&token.access_token, q.as_deref()).await {
                Ok(list) => {
                    let total = list.messages.len();
                    set_task(Box::new(move |t| t.total = total));
                    let mut fetched = Vec::with_capacity(total);
                    for (idx, msg_ref) in list.messages.iter().enumerate() {
                        match gmail_get_message(&token.access_token, &msg_ref.id).await {
                            Ok(msg) => fetched.push(gmail_msg_to_email(&msg)),
                            Err(e) => {
                                leptos::logging::log!("Failed to fetch message {}: {e}", msg_ref.id)
                            }
                        }
                        let done = idx + 1;
                        set_task(Box::new(move |t| t.fetched = done));
                    }
                    fetched.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
                    // Merge into the per-account cache (dedupe by Gmail id).
                    email_cache.update(|cache| {
                        let entry = cache.entry(token.email.clone()).or_insert_with(Vec::new);
                        for e in fetched {
                            if e.msg_id.is_empty() || !entry.iter().any(|x| x.msg_id == e.msg_id) {
                                entry.push(e);
                            }
                        }
                        entry.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
                    });
                    // Show immediately if this account is the active one.
                    let is_active = match active_account.get() {
                        Some(a) => a.email == token.email,
                        None => false,
                    };
                    if is_active {
                        if let Some(cached) = email_cache.get().get(&token.email) {
                            emails_store.set(cached.clone());
                        }
                    }
                    set_task(Box::new(|t| t.status = TaskStatus::Done));
                    // Auto-hide the finished bar after a short beat.
                    sleep(3000).await;
                    fetch_tasks
                        .update(|ts| ts.retain(|t| !(t.key == key && t.status == TaskStatus::Done)));
                }
                Err(e) => {
                    leptos::logging::log!("Failed to list messages: {e}");
                    fetch_error.set(Some(format!("{}: {e}", token.email)));
                    let msg = e.clone();
                    set_task(Box::new(move |t| t.status = TaskStatus::Error(msg)));
                    sleep(6000).await;
                    fetch_tasks.update(|ts| ts.retain(|t| !matches!(t.status, TaskStatus::Error(_))));
                }
            }
        });
    };


    // Sidebar account click: switch instantly from cache, else fetch.
    let on_account_click = move |token: OAuthToken| {
        active_account.set(Some(token.clone()));
        selected_id.set(None);
        active_folder.set(Folder::Inbox);
        if let Some(cached) = email_cache.get().get(&token.email) {
            emails_store.set(cached.clone());
        } else {
            spawn_fetch(token, None);
        }
    };

    // Fetch-panel "Fetch Range" button: one task per selected account+range.
    let on_fetch_range = move |from: String, to: String, account: String| {
        if from.is_empty() && to.is_empty() {
            fetch_error.set(Some("Pick at least one date for the range.".into()));
            return;
        }
        if let (Some(f), Some(t)) = (date_input_to_ms(&from), date_input_to_ms(&to)) {
            if f > t {
                fetch_error.set(Some("The from date is after the to date.".into()));
                return;
            }
        }
        let targets: Vec<OAuthToken> = {
            let all = connected_accounts.get();
            if account.is_empty() {
                all
            } else {
                all.into_iter().filter(|t| t.email == account).collect()
            }
        };
        if targets.is_empty() {
            fetch_error.set(Some(
                "No connected accounts to fetch — connect one in Settings first.".into(),
            ));
            return;
        }
        for token in targets {
            spawn_fetch(token, Some((from.clone(), to.clone())));
        }
    };

    // Detect OAuth callback on mount and route to Settings tab.
    let href = web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_default();
    let is_oauth_callback = href.contains("code=") || href.contains("/auth/callback/google");
    let initial_tab = if is_oauth_callback {
        Tab::Settings
    } else {
        Tab::Mailbox
    };
    let (active_tab, set_active_tab) = create_signal(initial_tab);

    // Handle OAuth callback after mount.
    if is_oauth_callback {
        let set_active_tab = set_active_tab.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match crate::oauth::handle_google_callback().await {
                Ok((token, _user_info)) => {
                    leptos::logging::log!("OAuth callback: got token for {}", token.email);
                    // Save token to sessionStorage so TokenVault can pick it up and persist to vault.
                    if let Ok(Some(storage)) = web_sys::window().unwrap().session_storage() {
                        let _ = storage.set_item(
                            "pending_oauth_token",
                            &serde_json::to_string(&token).unwrap_or_default(),
                        );
                    }
                    set_active_tab.set(Tab::Settings);
                }
                Err(e) => leptos::logging::log!("OAuth callback error: {e}"),
            }
        });
    }


    view! {
        <div class="app-container" class:drawer-open=move || drawer_open.get()>
            <Show when=move || !splash_done.get()>
                <Splash/>
            </Show>
            <TopBar query=query on_toggle_drawer=move || set_drawer_open.update(|d| *d = !*d)/>
            <div class="app-body">
                <Sidebar
                    active_folder=active_folder
                    emails=emails_store
                    connected_accounts=connected_accounts
                    email_cache=email_cache
                    set_active_tab=set_active_tab
                    set_drawer_open=set_drawer_open
                    on_oauth_account_click=on_account_click
                />
                <div class="drawer-backdrop" on:click=move |_| set_drawer_open.set(false)></div>
                <main class="content">
                    <Show when=move || active_tab.get() == Tab::Mailbox>
                        <FetchPanel
                            connected_accounts=connected_accounts
                            tasks=fetch_tasks
                            error=fetch_error
                            on_fetch_range=on_fetch_range
                        />
                        <div class="mail-layout">
                            <EmailList
                                emails=emails_store
                                active_folder=active_folder
                                filter=list_filter
                                query=query
                                selected_id=selected_id
                                on_select=select_email
                            />
                            <ReadingPane emails=emails_store selected_id=selected_id/>
                        </div>
                    </Show>
                    <Show when=move || active_tab.get() == Tab::Tokens>
                        <TokensPage connected_accounts=connected_accounts/>
                    </Show>
                    <Show when=move || active_tab.get() == Tab::Settings>
                        <Settings connected_accounts=connected_accounts/>
                    </Show>
                    <Show when=move || active_tab.get() == Tab::About>
                        <About/>
                    </Show>
                </main>
            </div>
            <BottomNavigation active=active_tab set_active_tab=set_active_tab/>
        </div>
    }
}

