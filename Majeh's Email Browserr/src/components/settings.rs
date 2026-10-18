use leptos::*;
use crate::components::icons::Icon;
use crate::tokens::{OAuthToken, TokenVault};

/// Settings view: iOS-style grouped rows (Account, Notifications, Appearance, ...).
#[component]
pub fn Settings(connected_accounts: RwSignal<Vec<OAuthToken>>) -> impl IntoView {
    let (notifications, set_notifications) = create_signal(true);

    // Google OAuth client secret management (stored in localStorage, never committed).
    let client_secret = create_rw_signal(String::new());
    let secret_saved = create_rw_signal(false);
    let secret_error = create_rw_signal(String::new());

    // Check localStorage for an existing secret on mount.
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            if let Ok(Some(secret)) = storage.get_item("oauth_client_secret") {
                if !secret.is_empty() {
                    client_secret.set(secret.clone());
                    secret_saved.set(true);
                }
            }
        }
    });

    let save_secret = {
        let client_secret = client_secret.clone();
        let secret_saved = secret_saved.clone();
        let secret_error = secret_error.clone();
        move |_| {
            let secret = client_secret.get();
            if secret.is_empty() {
                secret_error.set("Client secret cannot be empty".into());
                return;
            }
            match web_sys::window().unwrap().local_storage() {
                Ok(Some(storage)) => {
                    match storage.set_item("oauth_client_secret", &secret) {
                        Ok(_) => {
                            secret_saved.set(true);
                            secret_error.set("".into());
                            leptos::logging::log!("Google client secret saved to localStorage");
                        }
                        Err(e) => secret_error.set(format!("Failed to save: {e:?}")),
                    }
                }
                Ok(None) => secret_error.set("localStorage not available".into()),
                Err(e) => secret_error.set(format!("localStorage error: {e:?}")),
            }
        }
    };

    let clear_secret = {
        let client_secret = client_secret.clone();
        let secret_saved = secret_saved.clone();
        let secret_error = secret_error.clone();
        move |_| {
            client_secret.set(String::new());
            secret_saved.set(false);
            secret_error.set("".into());
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.remove_item("oauth_client_secret");
            }
        }
    };

    let rows = [
        ("sun", "Appearance", "System"),
        ("refresh", "Email Sync", "Real-time"),
        ("tag", "Manage Labels", ""),
        ("shield", "Privacy & Security", ""),
        ("link", "Connected Apps", ""),
    ];

    view! {
        <div class="page-scroll">
            <div class="page-card settings-view">
                <h1>"Settings"</h1>
                <TokenVault connected_accounts=connected_accounts/>
                <ul class="settings-list">
                    <li class="settings-row">
                        <span class="row-icon blue"><Icon name="user"/></span>
                        <div class="row-text">
                            <span class="row-title">"Account"</span>
                            <span class="row-sub">"majeh@email.com"</span>
                        </div>
                        <Icon name="chevron-right" class="chevron"/>
                    </li>
                    <li class="settings-row">
                        <span class="row-icon amber"><Icon name="bell"/></span>
                        <div class="row-text">
                            <span class="row-title">"Notifications"</span>
                            <span class="row-sub">"Email, sound, badges"</span>
                        </div>
                        <button
                            class="toggle"
                            class:on=move || notifications.get()
                            on:click=move |_| set_notifications.update(|n| *n = !*n)
                            aria-label="Toggle notifications"
                        >
                            <span class="knob"></span>
                        </button>
                    </li>
                    {rows.into_iter().map(|(icon, title, sub)| view! {
                        <li class="settings-row">
                            <span class="row-icon slate"><Icon name=icon/></span>
                            <div class="row-text">
                                <span class="row-title">{title}</span>
                                {if sub.is_empty() { None } else { Some(view! { <span class="row-sub">{sub}</span> }) }}
                            </div>
                            <Icon name="chevron-right" class="chevron"/>
                        </li>
                    }).collect_view()}
                    <li class="settings-row">
                        <span class="row-icon blue"><Icon name="login"/></span>
                        <div class="row-text">
                            <span class="row-title">"Google Account"</span>
                            <span class="row-sub">"Connect email to load your inbox"</span>
                        </div>
                        <button
                            class="btn-primary"
                            on:click=move |_| {
                            let _ = crate::oauth::start_google_oauth();
                        }
                        >
                            <Icon name="login"/>"Connect Gmail"
                        </button>
                    </li>
                    <li class="settings-row">
                        <span class="row-icon slate"><Icon name="key"/></span>
                        <div class="row-text">
                            <span class="row-title">"Google Client Secret"</span>
                            <span class="row-sub">
                                {if secret_saved.get() { "Saved in browser" } else { "Required for OAuth token exchange" }}
                            </span>
                        </div>
                        <div class="secret-input-wrap">
                            <input
                                type="password"
                                placeholder="Enter Google OAuth client secret"
                                prop:value=move || client_secret.get()
                                on:input=move |ev| client_secret.set(event_target_value(&ev))
                                disabled=move || secret_saved.get()
                            />
                            {if secret_saved.get() {
                                view! {
                                    <button class="btn-secondary sm" on:click=clear_secret>"Change"</button>
                                }
                            } else {
                                view! {
                                    <button
                                        class="btn-primary sm"
                                        on:click=save_secret
                                        disabled=move || client_secret.get().is_empty()
                                    >
                                        "Save"
                                    </button>
                                }
                            }}
                            {if !secret_error.get().is_empty() {
                                view! {
                                    <span class="error-text">{secret_error.get()}</span>
                                }
                            } else {
                                view! { <span></span> }
                            }}
                        </div>
                    </li>
                </ul>
            </div>
        </div>
    }
}
