use leptos::*;
use crate::components::icons::Icon;

/// Top application bar: logo, global search, hamburger (mobile), profile chip.
#[component]
pub fn TopBar(
    query: RwSignal<String>,
    on_toggle_drawer: impl Fn() + Copy + 'static,
) -> impl IntoView {
    view! {
        <header class="top-bar">
            <button class="hamburger mobile-only" on:click=move |_| on_toggle_drawer() aria-label="Menu">
                <Icon name="menu"/>
            </button>
            <div class="brand">
                <div class="brand-logo"></div>
                <span class="brand-name">"Majeh's Email Browserr"</span>
            </div>
            <div class="search-wrap">
                <Icon name="search" class="search-icon"/>
                <input
                    class="search-input"
                    type="text"
                    placeholder="Search emails, people, or attachments..."
                    prop:value=move || query.get()
                    on:input=move |ev| query.set(event_target_value(&ev))
                />
            </div>
            <div class="top-bar-right">
                <button class="icon-btn" aria-label="Notifications"><Icon name="bell"/></button>
                <div class="profile-chip">
                    <div class="avatar avatar-blue">"M"</div>
                    <div class="profile-meta">
                        <span class="profile-name">"Majeh"</span>
                        <span class="profile-status">"Productive Today"</span>
                    </div>
                </div>
            </div>
        </header>
    }
}
