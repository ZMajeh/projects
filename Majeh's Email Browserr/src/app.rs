use leptos::*;
use serde_json;
use crate::components::{
    about::About,
    email_list::{EmailList, ListFilter},
    navigation::{BottomNavigation, Tab},
    reading_pane::ReadingPane,
    settings::Settings,
    sidebar::Sidebar,
    top_bar::TopBar,
};
use crate::data::{emails, Folder, Email};
use crate::tokens::OAuthToken;
use crate::oauth::{gmail_list_messages, gmail_get_message};

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
    // Loading state for email fetching.
    let loading = create_rw_signal(false);

    // Selecting an email marks it read and (on mobile) opens the reading pane.
    let select_email = move |id: usize| {
        emails_store.update(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.id == id) {
                e.unread = false;
            }
        });
        selected_id.set(Some(id));
    };

    // Fetch emails for a connected OAuth account via Gmail API.
    let fetch_emails_for_account = {
        let emails_store = emails_store.clone();
        let active_account = active_account.clone();
        let loading = loading.clone();
        move |token: OAuthToken| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                active_account.set(Some(token.clone()));
                let access_token = token.access_token.clone();
                match gmail_list_messages(&access_token).await {
                    Ok(list) => {
                        let mut fetched_emails = Vec::new();
                        for (idx, msg_ref) in list.messages.iter().enumerate() {
                            match gmail_get_message(&access_token, &msg_ref.id).await {
                                Ok(msg) => {
                                    let subject = msg
                                        .payload
                                        .as_ref()
                                        .and_then(|p| {
                                            p.headers.iter().find(|h| h.name == "Subject")
                                        })
                                        .map(|h| h.value.clone())
                                        .unwrap_or_else(|| "(No Subject)".into());
                                    let sender = msg
                                        .payload
                                        .as_ref()
                                        .and_then(|p| {
                                            p.headers.iter().find(|h| h.name == "From")
                                        })
                                        .map(|h| h.value.clone())
                                        .unwrap_or_else(|| "Unknown".into());
                                    let preview = msg.snippet.clone();
                                    let time = "".into();
                                    let initial = sender
                                        .chars()
                                        .next()
                                        .unwrap_or('?')
                                        .to_uppercase()
                                        .next()
                                        .unwrap_or('?')
                                        .to_string();
                                    let color = "#4285f4".into();
                                    let unread = msg.label_ids.iter().any(|l| l == "UNREAD");
                                    let starred = msg.label_ids.iter().any(|l| l == "STARRED");
                                    let body = vec![preview.clone()];
                                    fetched_emails.push(Email {
                                        id: idx,
                                        sender,
                                        address: "".into(),
                                        subject,
                                        preview,
                                        time,
                                        initial,
                                        color,
                                        unread,
                                        starred,
                                        folder: Folder::Inbox,
                                        body,
                                    });
                                }
                                Err(e) => leptos::logging::log!("Failed to fetch message {}: {}", msg_ref.id, e),
                            }
                        }
                        emails_store.set(fetched_emails);
                    }
                    Err(e) => leptos::logging::log!("Failed to list messages: {e}"),
                }
                loading.set(false);
            });
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
                        let _ = storage.set_item("pending_oauth_token", &serde_json::to_string(&token).unwrap_or_default());
                    }
                    set_active_tab.set(Tab::Settings);
                }
                Err(e) => leptos::logging::log!("OAuth callback error: {e}"),
            }
        });
    }

    view! {
        <div class="app-container" class:drawer-open=move || drawer_open.get()>
            <TopBar query=query on_toggle_drawer=move || set_drawer_open.update(|d| *d = !*d)/>
            <div class="app-body">
                <Sidebar
                    active_folder=active_folder
                    emails=emails_store
                    connected_accounts=connected_accounts
                    set_active_tab=set_active_tab
                    set_drawer_open=set_drawer_open
                    on_oauth_account_click=fetch_emails_for_account
                    loading=loading
                />
                <div class="drawer-backdrop" on:click=move |_| set_drawer_open.set(false)></div>
                <main class="content">
                    <Show when=move || active_tab.get() == Tab::Mailbox>
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

