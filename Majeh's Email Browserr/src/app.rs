use leptos::*;
use crate::components::{
    about::About,
    email_list::{EmailList, ListFilter},
    fetch_panel::FetchPanel,
    home::Home,
    navigation::{BottomNavigation, Tab},
    reading_pane::ReadingPane,
    settings::Settings,
    sidebar::Sidebar,
    splash::Splash,
    tokens_page::TokensPage,
    top_bar::TopBar,
};
use crate::util::sleep;
use crate::data::{emails, format_email_time, init_email_id_counter, load_email_cache, next_email_id, save_email_cache, Email, FetchTask, Folder, TaskStatus};
use crate::oauth::{
    config, gmail_extract_body, gmail_get_message, gmail_list_messages_query,
    parse_address_header, refresh_access_token, DEFAULT_MAX_RESULTS, MAX_RESULTS_LIMIT,
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

/// Unique key for a fetch task: account + date range + requested count.
fn task_key(email: &str, from: &str, to: &str, max_results: u32) -> String {
    format!("{email}|{from}|{to}|{max_results}")
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
    let email_cache = create_rw_signal(load_email_cache());
    // Persistent email id counter — seeded from localStorage on mount.
    init_email_id_counter();
    // In-flight/finished fetch jobs, each with its own progress bar.
    let fetch_tasks = create_rw_signal(Vec::<FetchTask>::new());
    // Last user-facing fetch/OAuth error (dismissed by the user).
    let fetch_error = create_rw_signal(Option::<String>::None);

    // Startup diagnostics — a snapshot of everything that decides whether an
    // account can be added and shown. Open the browser console and look for
    // the `[majeh]` lines to see exactly where a flow stopped.
    wasm_bindgen_futures::spawn_local(async move {
        crate::util::log_info("--- app startup ---");
        crate::util::log_kv(
            "vault exists (localStorage)",
            &crate::tokens::vault_exists().to_string(),
        );
        crate::util::log_kv(
            "vault unlocked (this tab)",
            &crate::tokens::vault_is_unlocked().to_string(),
        );
        crate::util::log_kv(
            "accounts loaded in memory",
            &connected_accounts.get_untracked().len().to_string(),
        );
        match crate::oauth::config() {
            Ok((id, secret, redirect)) => {
                crate::util::log_kv("oauth client id", &id);
                crate::util::log_kv(
                    "oauth client secret",
                    if secret.is_empty() { "MISSING" } else { "set" },
                );
                crate::util::log_kv("oauth redirect uri", &redirect);
            }
            Err(e) => crate::util::log_error(&format!("oauth config is incomplete: {e}")),
        }
        let pending = crate::tokens::pending_token_count();
        crate::util::log_kv("pending (unsaved) oauth tokens", &pending.to_string());
        crate::util::log_info("--- end startup ---");
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
                    save_email_cache(&email_cache.get());
                });
            }
        }
    });


    // Fetch emails for an account via the Gmail API, with a per-task progress
    // bar. `range = Some((from, to))` applies a manual date filter (Gmail
    // `after:`/`before:` query); `None` fetches the latest inbox messages.
    let spawn_fetch = move |token: OAuthToken, range: Option<(String, String)>, limit: Option<u32>| {
        let from = range.as_ref().map(|r| r.0.clone()).unwrap_or_default();
        let to = range.as_ref().map(|r| r.1.clone()).unwrap_or_default();
        // `None` = auto (the current default of 20); `Some(n)` = fetch n emails.
        let max_results = limit.unwrap_or(DEFAULT_MAX_RESULTS).clamp(1, MAX_RESULTS_LIMIT);
        let label = if from.is_empty() && to.is_empty() {
            if limit.is_some() {
                format!("max {max_results}")
            } else {
                String::new()
            }
        } else {
            format!(
                "{} → {}{}",
                if from.is_empty() { "…" } else { &from },
                if to.is_empty() { "…" } else { &to },
                if limit.is_some() { format!(" · max {max_results}") } else { String::new() }
            )
        };
        let key = task_key(&token.email, &from, &to, max_results);
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
            // Persist the fresh token to the encrypted vault so a page reload
            // restores it instead of yesterday's stored copy.
            crate::tokens::persist_vault_now(&connected_accounts.get_untracked());
            // Auto-persist the refreshed tokens to the encrypted vault so a
            // page reload restores them instead of rolling back to the stale
            // stored copy (no-op while the vault is locked).
            crate::tokens::persist_vault_now(&connected_accounts.get_untracked());
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

            match gmail_list_messages_query(&token.access_token, q.as_deref(), max_results).await {
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
                    save_email_cache(&email_cache.get());
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
            spawn_fetch(token, None, None);
        }
    };

    // Mailbox "Refresh" button: re-fetch the active OAuth account's inbox and
    // merge new messages into the cache (dedupe by `msg_id`). Unlike a plain
    // account click, this always hits the network — the cache never expires
    // on its own. Refresh is a no-op for mock (non-OAuth) accounts and while
    // the same account's task is already running.
    let on_refresh_inbox = move || {
        let Some(acc) = active_account.get() else {
            return;
        };
        if !connected_accounts
            .get()
            .iter()
            .any(|t| t.email == acc.email)
        {
            return;
        }
        let inbox_key = task_key(&acc.email, "", "", DEFAULT_MAX_RESULTS);
        let already_running = fetch_tasks
            .get()
            .iter()
            .any(|t| t.key == inbox_key && matches!(t.status, TaskStatus::Running));
        if already_running {
            return;
        }
        spawn_fetch(acc, None, None);
    };

    // Whether the active account's inbox fetch is running right now (drives
    // the Refresh button's spinner/disabled state in the mailbox header).
    let inbox_refreshing = move || {
        let Some(acc) = active_account.get() else {
            return false;
        };
        let inbox_key = task_key(&acc.email, "", "", DEFAULT_MAX_RESULTS);
        fetch_tasks
            .get()
            .iter()
            .any(|t| t.key == inbox_key && matches!(t.status, TaskStatus::Running))
    };

    // Fetch-panel "Fetch Range" button: one task per selected account+range.
    let on_fetch_range = move |from: String, to: String, account: String, limit: String| {
        if from.is_empty() && to.is_empty() {
            fetch_error.set(Some("Pick at least one date for the range.".into()));
            return;
        }
        // Empty count = auto (current behaviour: up to 20 emails). A number
        // caps how many emails are pulled for that range.
        let requested: Option<u32> = if limit.trim().is_empty() {
            None
        } else {
            match limit.trim().parse::<u32>() {
                Ok(n) if n == 0 => {
                    fetch_error.set(Some("Number of emails must be at least 1.".into()));
                    return;
                }
                Ok(n) => Some(n.min(MAX_RESULTS_LIMIT)),
                Err(_) => {
                    fetch_error.set(Some("Number of emails must be a whole number.".into()));
                    return;
                }
            }
        };
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
            spawn_fetch(token, Some((from.clone(), to.clone())), requested);
        }
    };

    // Merge any OAuth token that arrived while the vault was locked into the
    // account list and persist the vault. This lives at the app level (not
    // inside the Settings tab) so it runs no matter which tab is open.
    // The pending tokens live in sessionStorage, so they survive a page reload.
    create_effect(move |_| {
        // Track the account list so this re-runs when it changes (which
        // includes the moment the vault is unlocked and its accounts load).
        let _accounts = connected_accounts.get();
        if !crate::tokens::vault_is_unlocked() {
            return;
        }
        let Some(session_pw) = crate::tokens::vault_password() else {
            return;
        };
        let pending = crate::tokens::take_pending_tokens();
        if pending.is_empty() {
            return;
        }
        let mut current = connected_accounts.get_untracked();
        let mut added = 0usize;
        for token in pending {
            let email = token.email.clone();
            current.retain(|t| t.email != email);
            current.push(token);
            added += 1;
        }
        connected_accounts.set(current.clone());
        // Persist the full list (old + new) with the session password through
        // the single choke point.
        match crate::tokens::save_vault(&session_pw, &current) {
            Ok(()) => {
                crate::util::log_info(&format!("saved {added} pending account(s) to the vault"));
            }
            Err(e) => crate::util::log_error(&format!("vault persist failed: {e}")),
        }
    });

    // Detect OAuth callback on mount and route to Settings tab.
    let href = web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_default();
    let is_oauth_callback = href.contains("code=") || href.contains("/auth/callback/google");
    let initial_tab = if is_oauth_callback {
        Tab::Settings
    } else {
        Tab::Home
    };
    let (active_tab, set_active_tab) = create_signal(initial_tab);

    // One shared "add a Google account" entry point (sidebar "Add Account",
    // the Accounts "+" button and Settings -> Connect Gmail). Failures are
    // surfaced in a global banner so they are visible on every tab.
    let oauth_error = create_rw_signal(Option::<String>::None);
    // The popup writes its result into localStorage; this listener picks it up
    // instantly when the browser kept the `window.opener` link alive.
    if let Err(e) = crate::oauth::install_popup_listener() {
        crate::util::log_error(&format!("could not install the popup listener: {e}"));
    }

    // Add a freshly obtained token to the shared account list and persist it.
    // The vault must be unlocked to write it; when it is locked the token is
    // stashed and merged into the vault on the next unlock.
    let add_account = move |token: OAuthToken| {
        let email = token.email.clone();
        let mut list = connected_accounts.get_untracked();
        // Replace an existing entry for the same address (re-connecting an
        // account refreshes its tokens).
        list.retain(|t| t.email != email);
        list.push(token.clone());
        connected_accounts.set(list.clone());
        if crate::tokens::vault_is_unlocked() {
            crate::tokens::persist_vault_now(&list);
            crate::util::log_info(&format!("added {email} to the vault"));
        } else {
            crate::tokens::stash_pending_token(&token);
            crate::util::log_info(&format!(
                "added {email} to the account list — unlock the vault to save it"
            ));
        }
    };

    let add_google_account = move || {
        // Prefer the popup flow: the app stays loaded, so the unlocked vault
        // and the account list survive the sign-in.
        match crate::oauth::start_google_oauth_popup() {
            Ok(popup) => {
                let add_account = add_account;
                let oauth_error = oauth_error;
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::oauth::await_popup_result(&popup, 300_000).await {
                        Ok(token) => add_account(token),
                        Err(e) => {
                            crate::util::log_error(&format!("Google sign-in failed: {e}"));
                            oauth_error.set(Some(format!(
                                "Google sign-in failed: {e}. Check Settings for the Google Client ID and Secret."
                            )));
                        }
                    }
                });
            }
            Err(e) => {
                // The popup was blocked - fall back to a full-page redirect.
                crate::util::log_error(&format!(
                    "could not open the sign-in popup ({e}) - falling back to a redirect"
                ));
                match crate::oauth::start_google_oauth() {
                    Ok(()) => {}
                    Err(e2) => oauth_error.set(Some(format!(
                        "Could not start Google sign-in: {e2}. Check Settings for the Google Client ID and Secret."
                    ))),
                }
            }
        }
    };

    // Handle OAuth callback after mount (the same-tab fallback flow).
    if is_oauth_callback {
        let set_active_tab = set_active_tab.clone();
        let add_account = add_account;
        wasm_bindgen_futures::spawn_local(async move {
            match crate::oauth::handle_google_callback().await {
                Ok((token, _user_info)) => {
                    crate::util::log_info(&format!(
                        "OAuth callback: got token for {}",
                        token.email
                    ));
                    add_account(token);
                    // Strip the single-use `?code=`/`state=` from the URL so a
                    // later browser refresh can't re-trigger this callback and
                    // re-consume (or fail on) the already-used auth code.
                    if let Some(window) = web_sys::window() {
                        if let Ok(path) = window.location().pathname() {
                            let _ = window
                                .history()
                                .map(|h| h.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path)));
                        }
                    }
                    set_active_tab.set(Tab::Settings);
                }
                Err(e) => {
                    crate::util::log_error(&format!("OAuth callback error: {e}"));
                    oauth_error.set(Some(format!("Google sign-in failed: {e}")));
                }
            }
        });
    }


    view! {
        <div class="app-container" class:drawer-open=move || drawer_open.get()>
            <Splash/>
            <TopBar query=query on_toggle_drawer=move || set_drawer_open.update(|d| *d = !*d)/>
            <div class="app-body">
                <Sidebar
                    active_tab=active_tab
                    active_folder=active_folder
                    emails=emails_store
                    connected_accounts=connected_accounts
                    email_cache=email_cache
                    set_active_tab=set_active_tab
                    set_drawer_open=set_drawer_open
                    on_oauth_account_click=on_account_click
                    on_add_account=add_google_account
                />
                <div class="drawer-backdrop" on:click=move |_| set_drawer_open.set(false)></div>
                <main class="content">
                    {move || {
                        oauth_error
                            .get()
                            .map(|e| view! {
                                <div class="fetch-error global-error">
                                    <span>{e}</span>
                                    <button
                                        class="icon-btn sm"
                                        aria-label="Dismiss"
                                        on:click=move |_| oauth_error.set(None)
                                    >
                                        "✕"
                                    </button>
                                </div>
                            })
                    }}
                    <Show when=move || active_tab.get() == Tab::Home>
                        <Home
                            connected_accounts=connected_accounts
                            vault_exists=crate::tokens::vault_exists()
                            set_active_tab=set_active_tab
                        />
                    </Show>
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
                                show_refresh=Callback::new(move |_| active_account.get().is_some())
                                refreshing=Callback::new(move |_| inbox_refreshing())
                                on_refresh=Callback::new(move |_| on_refresh_inbox())
                            />
                            <ReadingPane emails=emails_store selected_id=selected_id/>
                        </div>
                    </Show>
                    <Show when=move || active_tab.get() == Tab::Tokens>
                        <TokensPage connected_accounts=connected_accounts email_cache=email_cache emails_store=emails_store/>
                    </Show>
                    <Show when=move || active_tab.get() == Tab::Settings>
                        <Settings
                            connected_accounts=connected_accounts
                            on_connect_google=add_google_account
                        />
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

