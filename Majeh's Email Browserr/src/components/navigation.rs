use leptos::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Home,
    Mailbox,
    Tokens,
    Settings,
    About,
}

/// Bottom tab bar (mobile only; hidden by CSS on desktop).
#[component]
pub fn BottomNavigation(active: ReadSignal<Tab>, set_active_tab: WriteSignal<Tab>) -> impl IntoView {
    let items = [
        (Tab::Home, "home", "Home"),
        (Tab::Mailbox, "mail", "Mailbox"),
        (Tab::Tokens, "shield", "Tokens"),
        (Tab::Settings, "settings", "Settings"),
        (Tab::About, "info", "About"),
    ];

    view! {
        <nav class="bottom-nav mobile-only">
            {items.into_iter().map(|(tab, icon, label)| view! {
                <button
                    class="nav-item"
                    class:active=move || active.get() == tab
                    on:click=move |_| set_active_tab.set(tab)
                >
                    <crate::components::icons::Icon name=icon/>
                    <span>{label}</span>
                </button>
            }).collect_view()}
        </nav>
    }
}
