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
    let on_login = Callback::new(move |u| set_user.set(Some(u)));
    let on_logout = Callback::new(move |_| set_user.set(None));

    view! {
        <Router>
            <Routes>
                // Public Route
                <Route path="/login" view=move || view! { <Login on_login=on_login /> } />

                // Protected Routes inside DashboardLayout
                <Route 
                    path="/" 
                    view=move || {
                        if let Some(u) = user.get() {
                            view! { <DashboardLayout user=u on_logout=on_logout><Outlet/></DashboardLayout> }.into_view()
                        } else {
                            view! { <Redirect path="/login"/> }.into_view()
                        }
                    }
                >
                    <Route path="" view=DashboardHome />
                    <Route path="rooms" view=Rooms />
                    <Route path="customers" view=Customers />
                    <Route path="bookings" view=Bookings />
                </Route>
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
