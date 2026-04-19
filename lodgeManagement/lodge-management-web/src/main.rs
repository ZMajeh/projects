mod models;
mod api;
mod utils;
mod components;

use leptos::*;
use leptos_router::*;
use crate::utils::get_saved_user;
use crate::components::login::Login;
use crate::components::dashboard::DashboardLayout;

#[component]
fn App() -> impl IntoView {
    let (user, set_user) = create_signal(get_saved_user());

    view! {
        <Router>
            <main>
                {move || match user.get() {
                    Some(u) => view! { <DashboardLayout user=u on_logout=Callback::new(move |_| set_user.set(None))/> }.into_view(),
                    None => view! { <Login on_login=Callback::new(move |u| set_user.set(Some(u)))/> }.into_view(),
                }}
            </main>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
