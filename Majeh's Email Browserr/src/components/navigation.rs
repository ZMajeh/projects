use leptos::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Mailbox,
    Settings,
    About,
}

#[component]
pub fn BottomNavigation(
    active_tab: WriteSignal<Tab>,
    mailbox_visible: ReadSignal<bool>,
    settings_visible: ReadSignal<bool>,
    about_visible: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="bottom-nav">
            <Show when=move || mailbox_visible.get()>
                <button on:click=move |_| active_tab.set(Tab::Mailbox)>"Mailbox"</button>
            </Show>
            <Show when=move || settings_visible.get()>
                <button on:click=move |_| active_tab.set(Tab::Settings)>"Settings"</button>
            </Show>
            <Show when=move || about_visible.get()>
                <button on:click=move |_| active_tab.set(Tab::About)>"About"</button>
            </Show>
        </div>
    }
}
