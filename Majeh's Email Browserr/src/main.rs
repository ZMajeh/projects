use leptos::*;
mod components;
use components::{mailbox::Mailbox, settings::Settings, about::About, navigation::{BottomNavigation, Tab}};

#[component]
fn App() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal(Tab::Mailbox);
    let (mailbox_visible, _) = create_signal(true);
    let (settings_visible, _) = create_signal(true);
    let (about_visible, _) = create_signal(true);

    view! {
        <div class="app-container">
            <main>
                <Show when=move || active_tab.get() == Tab::Mailbox>
                    <Mailbox />
                </Show>
                <Show when=move || active_tab.get() == Tab::Settings>
                    <Settings />
                </Show>
                <Show when=move || active_tab.get() == Tab::About>
                    <About />
                </Show>
            </main>
            <BottomNavigation
                active_tab=set_active_tab
                mailbox_visible=mailbox_visible
                settings_visible=settings_visible
                about_visible=about_visible
            />
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
