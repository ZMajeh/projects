use leptos::*;
use crate::components::icons::Icon;
use crate::tokens::TokenVault;

/// Settings view: iOS-style grouped rows (Account, Notifications, Appearance, ...).
#[component]
pub fn Settings() -> impl IntoView {
    let (notifications, set_notifications) = create_signal(true);

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
                <TokenVault/>
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
                </ul>
            </div>
        </div>
    }
}
