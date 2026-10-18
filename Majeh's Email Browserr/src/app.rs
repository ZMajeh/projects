use leptos::*;
use crate::components::{
    about::About,
    email_list::{EmailList, ListFilter},
    navigation::{BottomNavigation, Tab},
    reading_pane::ReadingPane,
    settings::Settings,
    sidebar::Sidebar,
    top_bar::TopBar,
};
use crate::data::{emails, Folder};

/// Root application component: top bar, sidebar, content panes, bottom nav (mobile).
#[component]
pub fn App() -> impl IntoView {
    let emails_store = create_rw_signal(emails());
    let selected_id = create_rw_signal::<Option<usize>>(None);
    let active_folder = create_rw_signal(Folder::Inbox);
    let list_filter = create_rw_signal(ListFilter::All);
    let query = create_rw_signal(String::new());
    let (drawer_open, set_drawer_open) = create_signal(false);

    // Selecting an email marks it read and (on mobile) opens the reading pane.
    let select_email = move |id: usize| {
        emails_store.update(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.id == id) {
                e.unread = false;
            }
        });
        selected_id.set(Some(id));
    };

    // Sidebar folder click: switch to Mailbox tab and close the mobile drawer.

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
                    // The token will be saved to the vault via the Connect Gmail flow in Settings.
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
                <Sidebar active_folder=active_folder emails=emails_store set_active_tab=set_active_tab set_drawer_open=set_drawer_open/>
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
                        <Settings/>
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

