use leptos::*;
use std::collections::HashMap;
use crate::components::icons::Icon;
use crate::components::navigation::Tab;
use crate::data::{Email, Folder};
use crate::tokens::OAuthToken;

/// Clickable section header with a collapse chevron.
#[component]
fn SectionHead(
    title: &'static str,
    expanded: ReadSignal<bool>,
    set_expanded: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div
            class="section-head"
            class:collapsed=move || !expanded.get()
            role="button"
            tabindex="0"
            on:click=move |_| set_expanded.update(|e| *e = !*e)
        >
            <span>{title}</span>
            <Icon name="chevron-down" class="chevron"/>
        </div>
    }
}

/// Left sidebar: accounts, folders, labels, compose, settings/about links, quote.
#[component]
pub fn Sidebar(
    active_tab: ReadSignal<Tab>,
    active_folder: RwSignal<Folder>,
    emails: RwSignal<Vec<Email>>,
    connected_accounts: RwSignal<Vec<OAuthToken>>,
    set_active_tab: WriteSignal<Tab>,
    set_drawer_open: WriteSignal<bool>,
    on_oauth_account_click: impl Fn(OAuthToken) + Copy + 'static,
    /// Starts the Google OAuth flow to add a new account.
    on_add_account: impl Fn() + Copy + 'static,
    email_cache: RwSignal<HashMap<String, Vec<Email>>>,
) -> impl IntoView {
    let folders = Folder::ALL;
    let labels = [("Work", "#22c55e"), ("Personal", "#eab308"), ("Projects", "#3b82f6"), ("Newsletters", "#a855f7"), ("Important", "#ef4444")];

    let (show_accounts, set_show_accounts) = create_signal(true);
    let (show_mail, set_show_mail) = create_signal(true);
    let (show_labels, set_show_labels) = create_signal(true);

    let folder_counts = move || {
        let list = emails.get();
        move |f: Folder| list.iter().filter(|e| e.folder == f).count()
    };

    view! {
        <aside class="sidebar">
            <div class="sidebar-scroll">
                <button
                    class="folder-item home-item"
                    class:active=move || active_tab.get() == Tab::Home
                    on:click=move |_| { set_active_tab.set(Tab::Home); set_drawer_open.set(false); }
                >
                    <Icon name="home"/>
                    <span class="folder-label">"Home"</span>
                </button>

                <div
                    class="section-head"
                    class:collapsed=move || !show_accounts.get()
                    role="button"
                    tabindex="0"
                    on:click=move |_| set_show_accounts.update(|e| *e = !*e)
                >
                    <span>"Accounts"</span>
                    <button
                        class="icon-btn sm"
                        aria-label="Add account"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            set_drawer_open.set(false);
                            on_add_account();
                        }
                    >
                        <Icon name="plus"/>
                    </button>
                    <Icon name="chevron-down" class="chevron"/>
                </div>
                <Show when=move || show_accounts.get()>
                    <ul class="account-list">
                        {move || {
                            let connected = connected_accounts.get();
                            let cache = email_cache.get();
                            if connected.is_empty() {
                                return view! {
                                    <li class="account-empty">
                                        "No accounts yet — connect one in Settings."
                                    </li>
                                }.into_view();
                            }
                            connected.iter().map(|t| {
                                let initial = t.email.chars().next().unwrap_or('?').to_string();
                                let count = cache.get(&t.email).map(|v| v.len());
                                let name = t.email.clone();
                                let email_clone = t.email.clone();
                                let token = t.clone();
                                let set_tab = set_active_tab;
                                let on_oauth = on_oauth_account_click;
                                view! {
                                    <li class="account-item" on:click=move |_| {
                                        set_tab.set(Tab::Mailbox);
                                        on_oauth(token.clone());
                                    }>
                                        <div class="avatar" style="background:#4285f4">{initial}</div>
                                        <div class="account-meta">
                                            <span class="account-name">{name}</span>
                                            <span class="account-email">{email_clone}</span>
                                            {count.map(|n| view! {
                                                <span class="account-count">{format!("{n} emails")}</span>
                                            })}
                                        </div>
                                        <span class="oauth-badge">"OAuth"</span>
                                    </li>
                                }
                            }).collect_view().into_view()
                        }}
                    </ul>
                    <button
                        class="btn-add-account"
                        on:click=move |_| {
                            set_drawer_open.set(false);
                            on_add_account();
                        }
                    >
                        <Icon name="plus"/>"Add Account"
                    </button>
                </Show>

                <SectionHead title="Mail" expanded=show_mail set_expanded=set_show_mail/>
                <Show when=move || show_mail.get()>
                    <nav class="folder-list">
                        {folders.into_iter().map(|f| view! {
                            <button
                                class="folder-item"
                                class:active=move || {
                                    active_tab.get() == Tab::Mailbox && active_folder.get() == f
                                }
                                on:click=move |_| { active_folder.set(f); set_active_tab.set(Tab::Mailbox); set_drawer_open.set(false); }
                            >
                                <Icon name=f.icon()/>
                                <span class="folder-label">{f.label()}</span>
                                {if f == Folder::Inbox {
                                    let count = folder_counts()(Folder::Inbox);
                                    Some(view! { <span class="count">{count}</span> })
                                } else { None }}
                            </button>
                        }).collect_view()}
                    </nav>
                </Show>

                <SectionHead title="Labels" expanded=show_labels set_expanded=set_show_labels/>
                <Show when=move || show_labels.get()>
                    <ul class="label-list">
                        {labels.into_iter().map(|(name, color)| view! {
                            <li class="label-item">
                                <span class="label-dot" style=format!("background:{color}")></span>
                                {name}
                            </li>
                        }).collect_view()}
                        <li class="label-item add"><Icon name="plus"/>"Add Label"</li>
                    </ul>
                </Show>

                <div class="sidebar-footer">
                    <button
                        class="folder-item"
                        class:active=move || active_tab.get() == Tab::Tokens
                        on:click=move |_| { set_active_tab.set(Tab::Tokens); set_drawer_open.set(false); }
                    ><Icon name="shield"/>"Tokens"</button>
                    <button
                        class="folder-item"
                        class:active=move || active_tab.get() == Tab::Settings
                        on:click=move |_| { set_active_tab.set(Tab::Settings); set_drawer_open.set(false); }
                    ><Icon name="settings"/>"Settings"</button>
                    <button
                        class="folder-item"
                        class:active=move || active_tab.get() == Tab::About
                        on:click=move |_| { set_active_tab.set(Tab::About); set_drawer_open.set(false); }
                    ><Icon name="info"/>"About"</button>
                    <p class="quote">"A more organized you, a brighter tomorrow." <span>"— Majeh"</span></p>
                </div>
            </div>
        </aside>
    }
}

