use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use crate::models::{User, Room, Booking, NewBooking, NewRoom, Customer};
use crate::utils::{clear_user, wait_for_bridge};
use crate::api::{sign_out_user, add_booking_js, update_room_js};
use crate::components::rooms::{fetch_rooms, Rooms};
use crate::components::bookings::{fetch_bookings, Bookings};
use crate::components::customers::{fetch_customers, Customers};

#[component]
pub fn DashboardHome() -> impl IntoView {
    let (selected_date, set_selected_date) = create_signal(
        js_sys::Date::new_0().to_iso_string().as_string().unwrap()[..10].to_string()
    );
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (bookings, set_bookings) = create_signal(Vec::<Booking>::new());
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (loading, set_loading) = create_signal(true);

    let (show_book_modal, set_show_book_modal) = create_signal(None::<Room>);
    let (show_edit_modal, set_show_edit_modal) = create_signal(None::<Room>);

    let load_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_rooms.set(fetch_rooms().await);
            set_bookings.set(fetch_bookings().await);
            set_customers.set(fetch_customers("".to_string()).await);
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
                view! { <p style="text-align: center;">"Loading data..."</p> }.into_view()
            } else {
                view! {
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 20px;">
                        <For each=move || rooms.get() key=|r| r.id.clone().unwrap_or_default() children=move |r| {
                            let room_id = r.id.clone().unwrap_or_default();
                            let r_cloned = r.clone();
                            let r_cloned_2 = r.clone();
                            
                            let rid_style = room_id.clone();
                            let rid_status = room_id.clone();
                            let rid_label = room_id.clone();
                            let rid_btn = room_id.clone();
                            
                            view! {
                                <div style=move || {
                                    let occupied = is_occupied(&rid_style);
                                    format!("border: 1px solid #eee; border-radius: 12px; padding: 15px; text-align: center; background: #fff; border-top: 8px solid {};", 
                                        if occupied { "#e74c3c" } else { "#27ae60" }
                                    )
                                }>
                                    <strong style="font-size: 1.3rem; display: block; margin-bottom: 5px;">"Room " {r.number.clone()}</strong>
                                    <span style="font-size: 0.8rem; color: #7f8c8d; background: #f8f9fa; padding: 2px 8px; border-radius: 10px;">{r.room_type.clone()}</span>
                                    
                                    <div style=move || {
                                        let occupied = is_occupied(&rid_status);
                                        format!("margin: 15px 0; font-size: 0.8rem; font-weight: bold; color: {};", 
                                            if occupied { "#e74c3c" } else { "#27ae60" }
                                        )
                                    }>
                                        {move || if is_occupied(&rid_label) { "● OCCUPIED" } else { "● AVAILABLE" }}
                                    </div>

                                    <div style="display: flex; gap: 8px; margin-top: 10px;">
                                        <button 
                                            on:click=move |_| set_show_book_modal.set(Some(r_cloned.clone()))
                                            disabled=move || is_occupied(&rid_btn)
                                            style="flex: 1; padding: 8px; font-size: 0.75rem; background: #27ae60;"
                                        >
                                            "Book"
                                        </button>
                                        <button 
                                            on:click=move |_| set_show_edit_modal.set(Some(r_cloned_2.clone()))
                                            style="flex: 1; padding: 8px; font-size: 0.75rem; background: #3498db;"
                                        >
                                            "Edit"
                                        </button>
                                    </div>
                                </div>
                            }
                        } />
                    </div>
                }.into_view()
            }}

            // --- QUICK BOOKING MODAL ---
            {move || show_book_modal.get().map(|room| {
                let r_id = room.id.clone().unwrap_or_default();
                let r_num = room.number.clone();
                let (sel_cust, set_sel_cust) = create_signal("".to_string());
                let (check_out, set_check_out) = create_signal("".to_string());
                let (saving, set_saving) = create_signal(false);

                let handle_book = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default();
                    set_saving.set(true);
                    let c_id = sel_cust.get();
                    let rid = r_id.clone();
                    let rnum = r_num.clone();
                    let date = selected_date.get_untracked();
                    let cout = check_out.get();
                    let cust_opt = customers.get_untracked().into_iter().find(|c| c.id.as_deref() == Some(&c_id));
                    
                    if let Some(c) = cust_opt {
                        let new_booking = NewBooking {
                            room_id: rid, customer_id: c_id, customer_name: c.full_name,
                            room_number: rnum, check_in_date: date,
                            check_out_date: cout, status: "Checked-In".to_string(),
                        };
                        spawn_local(async move {
                            wait_for_bridge().await;
                            let js_val = serde_wasm_bindgen::to_value(&new_booking).unwrap();
                            let _ = add_booking_js(js_val).await;
                            set_show_book_modal.set(None);
                            load_data();
                        });
                    }
                };

                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;">
                        <div class="card" style="width: 100%; max-width: 450px; padding: 2rem;">
                            <h3>"Quick Check-in"</h3>
                            <form on:submit=handle_book>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div>
                                        <label style="font-size: 0.8rem; font-weight: bold;">"Room Number"</label>
                                        <input type="text" value=room.number.clone() disabled style="background: #eee;" />
                                    </div>
                                    <div>
                                        <label style="font-size: 0.8rem; font-weight: bold;">"Check-in Date"</label>
                                        <input type="text" value=selected_date.get() disabled style="background: #eee;" />
                                    </div>
                                    <div>
                                        <label style="font-size: 0.8rem; font-weight: bold;">"Select Guest"</label>
                                        <select on:change=move |ev| set_sel_cust.set(event_target_value(&ev)) required>
                                            <option value="">"Choose guest..."</option>
                                            {move || {
                                                customers.get().into_iter()
                                                    .map(|c| {
                                                        let cid = c.id.clone().unwrap_or_default();
                                                        let cname = c.full_name.clone();
                                                        let ver = if c.verified { "✅" } else { "⚠️" };
                                                        view! { <option value=cid>{cname} " " {ver}</option> }
                                                    })
                                                    .collect_view()
                                            }}
                                        </select>
                                    </div>
                                    <div>
                                        <label style="font-size: 0.8rem; font-weight: bold;">"Check-out Date"</label>
                                        <input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) required />
                                    </div>
                                </div>
                                <div style="display: flex; gap: 10px; margin-top: 25px;">
                                    <button type="submit" disabled=saving style="flex: 1; background: #27ae60;">
                                        {move || if saving.get() { "Saving..." } else { "Confirm" }}
                                    </button>
                                    <button type="button" on:click=move |_| set_show_book_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            })}

            // --- QUICK ROOM EDIT MODAL ---
            {move || show_edit_modal.get().map(|room| {
                let r_id = room.id.clone().unwrap_or_default();
                let r_num = room.number.clone();
                let r_status = room.status.clone();
                let (r_type, set_r_type) = create_signal(room.room_type.clone());
                let (saving, set_saving) = create_signal(false);

                let handle_update = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default();
                    set_saving.set(true);
                    let rid = r_id.clone();
                    let rnum = r_num.clone();
                    let rstat = r_status.clone();
                    let rtype = r_type.get();
                    let updated_room = NewRoom { number: rnum, room_type: rtype, status: rstat };
                    spawn_local(async move {
                        wait_for_bridge().await;
                        let js_val = serde_wasm_bindgen::to_value(&updated_room).unwrap();
                        let _ = update_room_js(rid, js_val).await;
                        set_show_edit_modal.set(None);
                        load_data();
                    });
                };

                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;">
                        <div class="card" style="width: 100%; max-width: 400px; padding: 2rem;">
                            <h3>"Edit Room"</h3>
                            <form on:submit=handle_update>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div>
                                        <label style="font-size: 0.8rem; font-weight: bold;">"Room Number"</label>
                                        <input type="text" value=room.number.clone() disabled style="background: #eee;" />
                                    </div>
                                    <div>
                                        <label style="font-size: 0.8rem; font-weight: bold;">"Category"</label>
                                        <select on:change=move |ev| set_r_type.set(event_target_value(&ev)) prop:value=r_type>
                                            <option value="Delux">"Delux"</option>
                                            <option value="AC">"AC"</option>
                                            <option value="non-AC">"non-AC"</option>
                                        </select>
                                    </div>
                                </div>
                                <div style="display: flex; gap: 10px; margin-top: 25px;">
                                    <button type="submit" disabled=saving style="flex: 1; background: #3498db;">
                                        {move || if saving.get() { "Saving..." } else { "Save" }}
                                    </button>
                                    <button type="button" on:click=move |_| set_show_edit_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            })}
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
