use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use crate::models::{User, Room, Booking};
use crate::utils::{clear_user, wait_for_bridge};
use crate::api::sign_out_user;
use crate::components::rooms::fetch_rooms;
use crate::components::bookings::fetch_bookings;
use crate::components::rooms::Rooms;
use crate::components::customers::Customers;
use crate::components::bookings::Bookings;

#[component]
pub fn DashboardHome() -> impl IntoView {
    let (selected_date, set_selected_date) = create_signal(
        js_sys::Date::new_0().to_iso_string().as_string().unwrap()[..10].to_string()
    );
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (bookings, set_bookings) = create_signal(Vec::<Booking>::new());
    let (loading, set_loading) = create_signal(true);

    let load_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_rooms.set(fetch_rooms().await);
            set_bookings.set(fetch_bookings().await);
            set_loading.set(false);
        });
    };
    create_effect(move |_| { load_data(); });

    let navigate_day = move |days: f64| {
        let current_ms = js_sys::Date::parse(&selected_date.get_untracked());
        let new_date = js_sys::Date::new(&JsValue::from_f64(current_ms + (days * 86400000.0)));
        set_selected_date.set(new_date.to_iso_string().as_string().unwrap()[..10].to_string());
    };

    let is_occupied = move |room_id: &str| {
        let date_str = selected_date.get();
        bookings.get().iter().any(|b| {
            b.room_id == room_id && 
            b.status == "Checked-In" &&
            date_str >= b.check_in_date && 
            date_str < b.check_out_date
        })
    };

    view! {
        <div class="card">
            <div style="display: flex; flex-direction: column; align-items: center; gap: 1rem; margin-bottom: 2rem;">
                <h2>"Lodge Occupancy Overview"</h2>
                <div style="display: flex; align-items: center; gap: 15px; background: #fff; padding: 10px 20px; border-radius: 50px;">
                    <button on:click=move |_| navigate_day(-1.0) style="background: none; color: var(--primary); font-weight: bold; font-size: 1.2rem; border: none; cursor: pointer;">"←"</button>
                    <input type="date" 
                        on:input=move |ev| set_selected_date.set(event_target_value(&ev))
                        prop:value=selected_date
                        style="border: none; font-weight: bold; cursor: pointer; text-align: center; width: auto; background: none;"
                    />
                    <button on:click=move |_| navigate_day(1.0) style="background: none; color: var(--primary); font-weight: bold; font-size: 1.2rem; border: none; cursor: pointer;">"→"</button>
                </div>
            </div>

            {move || if loading.get() {
                view! { <p style="text-align: center;">"Loading rooms..."</p> }.into_view()
            } else {
                view! {
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 15px;">
                        <For each=move || rooms.get() key=|r| r.id.clone().unwrap_or_default() children=move |r| {
                            let occupied = is_occupied(r.id.as_deref().unwrap_or(""));
                            let r_num = r.number.clone();
                            let r_type = r.room_type.clone();
                            view! {
                                <div style=format!("border: 1px solid #eee; border-radius: 12px; padding: 15px; text-align: center; background: #fff; border-left: 5px solid {};", 
                                    if occupied { "#e74c3c" } else { "#27ae60" }
                                )>
                                    <strong style="font-size: 1.2rem;">"Room " {r_num}</strong>
                                    <p style="font-size: 0.8rem; color: #666; margin: 5px 0;">{r_type}</p>
                                    
                                    <div style=format!("margin: 10px 0; font-size: 0.75rem; font-weight: bold; color: {};", 
                                        if occupied { "#e74c3c" } else { "#27ae60" }
                                    )>
                                        {if occupied { "● OCCUPIED" } else { "● AVAILABLE" }}
                                    </div>

                                    <div style="display: flex; gap: 5px; margin-top: 10px;">
                                        <div style="flex: 1;">
                                            <A href="/bookings">
                                                <button style="width: 100%; padding: 8px; font-size: 0.7rem; background: #27ae60;">"Book"</button>
                                            </A>
                                        </div>
                                        <div style="flex: 1;">
                                            <A href="/rooms">
                                                <button style="width: 100%; padding: 8px; font-size: 0.7rem; background: #3498db;">"Edit"</button>
                                            </A>
                                        </div>
                                    </div>
                                </div>
                            }
                        } />
                    </div>
                }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn DashboardLayout(user: User, on_logout: Callback<()>) -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);
    let handle_logout = move |_| { 
        clear_user(); 
        spawn_local(async move { 
            wait_for_bridge().await; 
            let _ = sign_out_user().await; 
            on_logout.call(()); 
        }); 
    };

    view! { 
        <div class="app-layout">
            <div class=move || format!("sidebar-overlay {}", if menu_open.get() { "show" } else { "" }) on:click=move |_| set_menu_open.set(false)></div>
            <nav class=move || format!("sidebar {}", if menu_open.get() { "open" } else { "" })>
                <h2>"Lodge Manager"</h2>
                <A href="" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active" exact=true>"Overview"</A>
                <A href="rooms" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Rooms"</A>
                <A href="customers" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Customers"</A>
                <A href="bookings" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Bookings"</A>
                <div style="margin-top: auto; padding-top: 1rem; border-top: 1px solid #444; font-size: 0.8rem;">
                    <p style="color: #bdc3c7; overflow: hidden; text-overflow: ellipsis;">{user.email}</p>
                    <button on:click=handle_logout style="background-color: #e74c3c; width: 100%; margin-top: 10px;">"Logout"</button>
                </div>
            </nav>
            <main class="content">
                <header class="mobile-header">
                    <button on:click=move |_| set_menu_open.update(|v| *v = !*v) style="background: none; color: black; font-size: 1.5rem; padding: 0;">"☰"</button>
                    <strong>"Lodge Manager"</strong>
                    <div style="width: 30px;"></div>
                </header>
                <Routes>
                    <Route path="" view=DashboardHome />
                    <Route path="rooms" view=Rooms />
                    <Route path="customers" view=Customers />
                    <Route path="bookings" view=Bookings />
                </Routes>
            </main>
        </div> 
    }
}
