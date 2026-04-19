mod models;
mod api;
mod utils;
mod components;

use leptos::*;
use leptos_router::*;
use crate::utils::get_saved_user;
use crate::components::login::Login;
use crate::components::dashboard::{DashboardLayout, DashboardHome};
use crate::components::rooms::Rooms;
use crate::components::customers::Customers;
use crate::components::bookings::Bookings;

#[component]
fn App() -> impl IntoView {
    let (user, set_user) = create_signal(get_saved_user());

    view! {
        <Router>
            <main>
                {move || match user.get() {
                    None => view! { <Login on_login=Callback::new(move |u| set_user.set(Some(u)))/> }.into_view(),
                    Some(u) => view! { 
                        <DashboardLayout user=u on_logout=Callback::new(move |_| set_user.set(None))>
                            <Routes>
                                <Route path="" view=DashboardHome />
                                <Route path="rooms" view=Rooms />
                                <Route path="customers" view=Customers />
                                <Route path="bookings" view=Bookings />
                            </Routes>
                        </DashboardLayout>
                    }.into_view(),
                }}
            </main>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
