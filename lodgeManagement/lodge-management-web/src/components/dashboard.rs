use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use crate::models::{User, Room, Booking, NewBooking, NewRoom, Customer, Payment};
use crate::utils::{clear_user, wait_for_bridge};
use crate::api::{sign_out_user, add_booking_js, update_room_js, update_booking_js, delete_booking_js};
use crate::components::rooms::{fetch_rooms, Rooms};
use crate::components::bookings::{fetch_bookings, Bookings};
use crate::components::customers::{fetch_customers, Customers, CustomerForm};

fn get_active_booking_helper(bookings: &[Booking], date_str: &str, room_id: &str) -> Option<Booking> {
    bookings.iter().find(|b| {
        b.room_id == room_id && 
        b.status == "Checked-In" &&
        date_str >= b.check_in_date && 
        date_str < b.check_out_date
    }).cloned()
}

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
    let (show_edit_room_modal, set_show_edit_room_modal) = create_signal(None::<Room>);
    let (show_manage_stay_modal, set_show_manage_stay_modal) = create_signal(None::<Booking>);
    let (show_add_guest_modal, set_show_add_guest_modal) = create_signal(false);
    let (confirm_cancel_stay, set_confirm_cancel_stay) = create_signal(false);

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

    let get_daily_revenue = move || {
        let date_str = selected_date.get();
        bookings.get().iter()
            .flat_map(|b| b.payments.iter())
            .filter(|p| p.date == date_str)
            .map(|p| p.amount)
            .sum::<f64>()
    };

    view! {
        <div class="card">
            <div style="display: flex; flex-direction: column; align-items: center; gap: 1rem; margin-bottom: 2rem;">
                <h2>"Lodge Occupancy Overview"</h2>
                <div style="display: flex; gap: 20px; align-items: center; width: 100%; justify-content: center; flex-wrap: wrap;">
                    <div style="display: flex; align-items: center; gap: 15px; background: #fff; padding: 10px 20px; border-radius: 50px; box-shadow: 0 2px 5px rgba(0,0,0,0.1);">
                        <button on:click=move |_| navigate_day(-1.0) style="background: none; color: var(--primary); font-weight: bold; font-size: 1.2rem; border: none; cursor: pointer;">"←"</button>
                        <input type="date" on:input=move |ev| set_selected_date.set(event_target_value(&ev)) prop:value=selected_date style="border: none; font-weight: bold; cursor: pointer; text-align: center; width: auto; background: none;" />
                        <button on:click=move |_| navigate_day(1.0) style="background: none; color: var(--primary); font-weight: bold; font-size: 1.2rem; border: none; cursor: pointer;">"→"</button>
                    </div>
                    <div style="background: #27ae60; color: white; padding: 10px 25px; border-radius: 50px; font-weight: bold; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
                        "Today's Revenue: ₹" {move || get_daily_revenue()}
                    </div>
                </div>
            </div>

            {move || if loading.get() { view! { <p style="text-align: center;">"Loading data..."</p> }.into_view() } else {
                view! {
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 20px;">
                        <For each=move || rooms.get() key=|r| r.id.clone().unwrap_or_default() children=move |r| {
                            let room_id = r.id.clone().unwrap_or_default();
                            let r_cloned = r.clone();
                            let rid_style = room_id.clone();
                            let rid_status = room_id.clone();
                            let rid_label = room_id.clone();
                            let rid_btn = room_id.clone();
                            let rid_edit = room_id.clone();
                            
                            view! {
                                <div style=move || {
                                    let booking_opt = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid_style);
                                    let border_color = if let Some(b) = booking_opt {
                                        let paid: f64 = b.payments.iter().map(|p| p.amount).sum();
                                        if b.total_amount > paid { "#f39c12" } else { "#e74c3c" }
                                    } else { "#27ae60" };
                                    format!("border: 1px solid #eee; border-radius: 12px; padding: 15px; text-align: center; background: #fff; border-top: 8px solid {};", border_color)
                                }>
                                    <strong style="font-size: 1.3rem; display: block; margin-bottom: 5px;">"Room " {r.number.clone()}</strong>
                                    <span style="font-size: 0.8rem; color: #7f8c8d; background: #f8f9fa; padding: 2px 8px; border-radius: 10px;">{r.room_type.clone()} " • ₹" {r.price}</span>
                                    <div style=move || {
                                        let booking_opt = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid_status);
                                        let text_color = if let Some(b) = booking_opt {
                                            let paid: f64 = b.payments.iter().map(|p| p.amount).sum();
                                            if b.total_amount > paid { "#f39c12" } else { "#e74c3c" }
                                        } else { "#27ae60" };
                                        format!("margin: 15px 0; font-size: 0.8rem; font-weight: bold; color: {};", text_color)
                                    }>
                                        {move || {
                                            if let Some(b) = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid_label) {
                                                let paid: f64 = b.payments.iter().map(|p| p.amount).sum();
                                                if b.total_amount > paid { format!("● PENDING ₹{}", b.total_amount - paid) } else { "● OCCUPIED".to_string() }
                                            } else { "● AVAILABLE".to_string() }
                                        }}
                                    </div>
                                    <div style="display: flex; gap: 8px; margin-top: 10px;">
                                        {let rb = r_cloned.clone(); view! { <button on:click=move |_| set_show_book_modal.set(Some(rb.clone())) disabled=move || get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid_btn).is_some() style="flex: 1; padding: 8px; font-size: 0.75rem; background: #27ae60;">"Book"</button> }}
                                        {let re = r_cloned.clone(); view! { <button on:click=move |_| { if let Some(booking) = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid_edit) { set_confirm_cancel_stay.set(false); set_show_manage_stay_modal.set(Some(booking)); } else { set_show_edit_room_modal.set(Some(re.clone())); } } style="flex: 1; padding: 8px; font-size: 0.75rem; background: #3498db;">"Edit"</button> }}
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
                let r_price = room.price;
                let (sel_cust, set_sel_cust) = create_signal("".to_string());
                let (final_price, set_final_price) = create_signal(r_price.to_string());
                let (paid_now, set_paid_now) = create_signal(r_price.to_string());
                let (check_out, set_check_out) = create_signal("".to_string());
                let (saving, set_saving) = create_signal(false);
                let (guest_search, set_guest_search) = create_signal("".to_string());
                let filtered_guests = move || {
                    let q = guest_search.get().to_lowercase();
                    customers.get().into_iter().filter(|c| c.full_name.to_lowercase().contains(&q) || c.aadhaar.contains(&q)).collect::<Vec<_>>()
                };
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
                        let total = final_price.get().parse::<f64>().unwrap_or(0.0);
                        let first_pay = paid_now.get().parse::<f64>().unwrap_or(0.0);
                        let new_booking = NewBooking {
                            room_id: rid, customer_id: c_id, customer_name: c.full_name,
                            room_number: rnum, check_in_date: date,
                            check_out_date: cout, status: "Checked-In".to_string(),
                            total_amount: total,
                            payments: vec![Payment { amount: first_pay, date: selected_date.get_untracked() }]
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
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h3>"Quick Check-in"</h3><button on:click=move |_| set_show_add_guest_modal.set(true) style="font-size: 0.7rem; background: #e67e22; padding: 5px 10px;">"+ New Guest"</button></div>
                            <form on:submit=handle_book>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div style="display: flex; gap: 10px;">
                                        <div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Room"</label><input type="text" value=r_num.clone() disabled style="background: #eee;" /></div>
                                        <div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Check-in"</label><input type="text" value=selected_date.get() disabled style="background: #eee;" /></div>
                                    </div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Search Guest"</label><input type="text" placeholder="Search..." on:input=move |ev| set_guest_search.set(event_target_value(&ev)) style="margin-bottom: 5px;" /><select on:change=move |ev| set_sel_cust.set(event_target_value(&ev)) required prop:value=sel_cust><option value="">"Choose guest..."</option>{move || filtered_guests().into_iter().map(|c| { let cid = c.id.clone().unwrap_or_default(); let phone = c.phone.clone(); view! { <option value=cid>{c.full_name.clone()} " (" {phone} ")" </option> } }).collect_view()}</select></div>
                                    <div style="display: flex; gap: 10px;">
                                        <div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Total Price"</label><input type="number" on:input=move |ev| set_final_price.set(event_target_value(&ev)) prop:value=final_price required /></div>
                                        <div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Paying Now"</label><input type="number" on:input=move |ev| set_paid_now.set(event_target_value(&ev)) prop:value=paid_now required /></div>
                                    </div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Estimated Check-out"</label><input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) required /></div>
                                </div>
                                <div style="display: flex; gap: 10px; margin-top: 25px;"><button type="submit" disabled=saving style="flex: 2; background: #27ae60;">"Confirm"</button><button type="button" on:click=move |_| set_show_book_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button></div>
                            </form>
                        </div>
                    </div>
                }
            })}

            // --- ADD GUEST POPUP ---
            {move || if show_add_guest_modal.get() {
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 4000; padding: 1rem;">
                        <div class="card" style="width: 100%; max-width: 500px; padding: 1rem; max-height: 90vh; overflow-y: auto;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h3>"Register New Guest"</h3><button on:click=move |_| set_show_add_guest_modal.set(false) style="background: none; color: black; font-size: 1.5rem;">"×"</button></div>
                            <CustomerForm editing_id=create_memo(|_| None) initial_data=None on_success=Callback::new(move |_| { set_show_add_guest_modal.set(false); load_data(); }) />
                        </div>
                    </div>
                }.into_view()
            } else { view! {}.into_view() }}

            // --- MANAGE STAY MODAL ---
            {move || show_manage_stay_modal.get().map(|booking| {
                let b_id = booking.id.clone().unwrap_or_default();
                let b_rid = booking.room_id.clone();
                let b_name = booking.customer_name.clone();
                let b_room = booking.room_number.clone();
                let b_in = booking.check_in_date.clone();
                let b_total = booking.total_amount;
                let b_paid: f64 = booking.payments.iter().map(|p| p.amount).sum();
                let balance = b_total - b_paid;
                let (check_out, set_check_out) = create_signal(booking.check_out_date.clone());
                let (extra_payment, set_extra_payment) = create_signal(balance.to_string());
                let (saving, set_saving) = create_signal(false);
                let b_data = booking.clone();
                
                let b_id_c = b_id.clone();
                let b_rid_c = b_rid.clone();
                let b_date_c = selected_date.get_untracked();

                let handle_update = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default();
                    set_saving.set(true);
                    let bid = b_id_c.clone();
                    let extra = extra_payment.get().parse::<f64>().unwrap_or(0.0);
                    let date = b_date_c.clone();
                    let mut updated_payments = b_data.payments.clone();
                    if extra > 0.0 { updated_payments.push(Payment { amount: extra, date }); }
                    let updated_booking = NewBooking {
                        room_id: b_data.room_id.clone(), customer_id: b_data.customer_id.clone(),
                        customer_name: b_data.customer_name.clone(), room_number: b_data.room_number.clone(),
                        check_in_date: b_data.check_in_date.clone(), check_out_date: check_out.get(),
                        status: b_data.status.clone(), total_amount: b_data.total_amount, payments: updated_payments,
                    };
                    spawn_local(async move {
                        wait_for_bridge().await;
                        let js_val = serde_wasm_bindgen::to_value(&updated_booking).unwrap();
                        let _ = update_booking_js(bid, js_val).await;
                        set_show_manage_stay_modal.set(None);
                        load_data();
                    });
                };
                
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;">
                        <div class="card" style="width: 100%; max-width: 450px; padding: 2rem;">
                            <h3>"Manage Guest Stay"</h3>
                            <div style="margin-bottom: 20px; text-align: left; background: #f8f9fa; padding: 1rem; border-radius: 8px;">
                                <p><strong>"Guest: "</strong> {b_name}</p>
                                <p><strong>"Room: "</strong> {b_room} " | " <strong>"Check-in: "</strong> {b_in}</p>
                                <p><strong>"Total: "</strong> "₹" {b_total} " | " <strong>"Balance: "</strong> "₹" {balance}</p>
                            </div>
                            <form on:submit=handle_update>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Collect Payment"</label><input type="number" on:input=move |ev| set_extra_payment.set(event_target_value(&ev)) prop:value=extra_payment /></div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Update Check-out"</label><input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) prop:value=check_out required /></div>
                                </div>
                                <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 25px;">
                                    <button type="submit" disabled=saving style="background: #3498db;">"Save Changes"</button>
                                    {move || if confirm_cancel_stay.get() {
                                        let bf = b_id_c.clone(); let rf = b_rid_c.clone();
                                        view! { <div style="background: #fee2e2; padding: 10px; border-radius: 8px; border: 1px solid #ef4444; margin-top: 10px;"><p style="color: #b91c1c; font-weight: bold; margin-bottom: 10px;">"Really cancel?"</p><div style="display: flex; gap: 5px;"><button type="button" on:click=move |_| { let bf2=bf.clone(); let rf2=rf.clone(); spawn_local(async move { wait_for_bridge().await; let _ = delete_booking_js(bf2, rf2).await; set_show_manage_stay_modal.set(None); load_data(); }); } style="background: #ef4444; flex: 1;">"YES"</button><button type="button" on:click=move |_| set_confirm_cancel_stay.set(false) style="background: #6c757d; flex: 1;">"No"</button></div></div> }.into_view()
                                    } else { view! { <button type="button" on:click=move |_| set_confirm_cancel_stay.set(true) style="background: #e67e22; margin-top: 10px;">"Cancel / Checkout"</button> }.into_view() }}
                                    <button type="button" on:click=move |_| set_show_manage_stay_modal.set(None) style="background: #6c757d; margin-top: 10px;">"Close"</button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            })}

            // --- QUICK ROOM SETTINGS MODAL ---
            {move || show_edit_room_modal.get().map(|room| {
                let r_id = room.id.clone().unwrap_or_default();
                let (r_type, set_r_type) = create_signal(room.room_type.clone());
                let (r_price, set_r_price) = create_signal(room.price.to_string());
                let (saving, set_saving) = create_signal(false);
                let r_num = room.number.clone();
                let handle_room_update = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default();
                    set_saving.set(true);
                    let rid = r_id.clone();
                    let rnum = r_num.clone();
                    let updated_room = NewRoom { number: rnum, room_type: r_type.get(), status: "Available".to_string(), price: r_price.get().parse::<f64>().unwrap_or(0.0) };
                    spawn_local(async move {
                        wait_for_bridge().await;
                        let js_val = serde_wasm_bindgen::to_value(&updated_room).unwrap();
                        let _ = update_room_js(rid, js_val).await;
                        set_show_edit_room_modal.set(None);
                        load_data();
                    });
                };
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;">
                        <div class="card" style="width: 100%; max-width: 400px; padding: 2rem;">
                            <h3>"Edit Room Settings"</h3>
                            <form on:submit=handle_room_update>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Room Number"</label><input type="text" value=r_num.clone() disabled style="background: #eee;" /></div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Category"</label><select on:change=move |ev| set_r_type.set(event_target_value(&ev)) prop:value=r_type><option value="Delux">"Delux"</option><option value="AC">"AC"</option><option value="non-AC">"non-AC"</option></select></div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Base Price"</label><input type="number" on:input=move |ev| set_r_price.set(event_target_value(&ev)) prop:value=r_price /></div>
                                </div>
                                <div style="display: flex; gap: 10px; margin-top: 25px;"><button type="submit" disabled=saving style="flex: 1; background: #3498db;">"Save"</button><button type="button" on:click=move |_| set_show_edit_room_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button></div>
                            </form>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

#[component]
pub fn DashboardLayout(user: User, on_logout: Callback<()>, children: Children) -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);
    let handle_logout = move |_| { clear_user(); spawn_local(async move { wait_for_bridge().await; let _ = sign_out_user().await; on_logout.call(()); }); };
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
                {children()}
            </main>
        </div> 
    }
}
