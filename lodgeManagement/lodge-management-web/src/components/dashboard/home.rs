use leptos::*;
use wasm_bindgen::prelude::*;
use crate::models::{Room, Booking, NewBooking, NewRoom, Customer, Payment, ExtraGuest};
use crate::utils::wait_for_bridge;
use crate::api::{add_booking_js, update_room_js, update_booking_js};
use crate::components::rooms::fetch_rooms;
use crate::components::bookings::fetch_bookings;
use crate::components::customers::{fetch_customers, CustomerForm};
use super::printable_bill::PrintableBill;

fn get_active_booking_helper(bookings: &[Booking], date_str: &str, room_id: &str) -> Option<Booking> {
    bookings.iter().find(|b| {
        b.room_id == room_id && 
        b.status == "Checked-In" &&
        date_str >= b.check_in_date.as_str() && 
        date_str < b.check_out_date.as_str()
    }).cloned()
}

#[component]
pub fn DashboardHome() -> impl IntoView {
    let (selected_date, set_selected_date) = create_signal(js_sys::Date::new_0().to_iso_string().as_string().unwrap()[..10].to_string());
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (bookings, set_bookings) = create_signal(Vec::<Booking>::new());
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (loading, set_loading) = create_signal(true);

    let (show_book_modal, set_show_book_modal) = create_signal(None::<Room>);
    let (show_edit_room_modal, set_show_edit_room_modal) = create_signal(None::<Room>);
    let (show_manage_stay_modal, set_show_manage_stay_modal) = create_signal(None::<Booking>);
    let (show_bill_modal, set_show_bill_modal) = create_signal(None::<(Booking, Option<Customer>)>);
    let (show_add_guest_modal, set_show_add_guest_modal) = create_signal(false);
    let (confirm_cancel_stay, set_confirm_cancel_stay) = create_signal(false);
    let (confirm_checkout, set_confirm_checkout) = create_signal(false);

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
        let date_val = selected_date.get_untracked();
        let current_ms = js_sys::Date::parse(&date_val);
        let new_date = js_sys::Date::new(&JsValue::from_f64(current_ms + (days * 86400000.0)));
        set_selected_date.set(new_date.to_iso_string().as_string().unwrap()[..10].to_string());
    };

    let get_daily_revenue = move || {
        let date_str = selected_date.get();
        bookings.get().iter().flat_map(|b| b.payments.iter()).filter(|p| p.date == date_str).map(|p| p.amount).sum::<f64>()
    };

    let guests_on_day = move || {
        let date_str = selected_date.get();
        bookings.get().into_iter()
            .filter(|b| {
                let is_present = date_str >= b.check_in_date && date_str <= b.check_out_date;
                is_present && (b.status == "Checked-In" || b.status == "Checked-Out")
            })
            .map(|b| {
                let cust = customers.get().into_iter().find(|c| c.id.as_deref() == Some(&b.customer_id));
                let paid: f64 = b.payments.iter().map(|p| p.amount).sum();
                (b, cust, paid)
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="card">
            {move || show_bill_modal.get().map(|(b, c)| {
                let on_close = Callback::new(move |_| set_show_bill_modal.set(None));
                view! { <PrintableBill booking=b customer=c on_close=on_close /> }
            })}

            <div class=move || if show_bill_modal.get().is_some() { "no-print" } else { "" }>
                <div style="display: flex; flex-direction: column; align-items: center; gap: 1rem; margin-bottom: 2rem;">
                <h2 style="color: var(--primary); font-size: 1.8rem; font-weight: 800;">"Anand Lodge Occupancy"</h2>
                <div style="display: flex; gap: 20px; align-items: center; width: 100%; justify-content: center; flex-wrap: wrap;">
                    <div style="display: flex; align-items: center; gap: 15px; background: #fff; padding: 10px 20px; border-radius: 50px; box-shadow: 0 2px 5px rgba(0,0,0,0.1);">
                        <button on:click=move |_| navigate_day(-1.0) style="background: none; color: var(--primary); font-weight: bold; font-size: 1.2rem; border: none; cursor: pointer;">"←"</button>
                        <input type="date" on:input=move |ev| set_selected_date.set(event_target_value(&ev)) prop:value=selected_date style="border: none; font-weight: bold; cursor: pointer; text-align: center; width: auto; background: none;" />
                        <button on:click=move |_| navigate_day(1.0) style="background: none; color: var(--primary); font-weight: bold; font-size: 1.2rem; border: none; cursor: pointer;">"→"</button>
                    </div>
                    <div style="background: #27ae60; color: white; padding: 10px 25px; border-radius: 50px; font-weight: bold; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">"Today's Revenue: ₹" {move || get_daily_revenue()}</div>
                </div>
            </div>

            {move || if loading.get() { view! { <p style="text-align: center;">"Loading data..."</p> }.into_view() } else {
                view! {
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 20px;">
                        <For each=move || rooms.get() key=|r| r.id.clone().unwrap_or_default() children=move |r| {
                            let r_cloned = r.clone();
                            let rid_fixed = r_cloned.id.clone().unwrap_or_default();
                            
                            view! {
                                <div style={
                                    let rid = rid_fixed.clone();
                                    move || {
                                        let b_opt = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid);
                                        let color = if let Some(b) = b_opt {
                                            let p: f64 = b.payments.iter().map(|pay| pay.amount).sum();
                                            if b.total_amount > p { "#f39c12" } else { "#e74c3c" }
                                        } else { "#27ae60" };
                                        format!("border: 1px solid #eee; border-radius: 12px; padding: 15px; text-align: center; background: #fff; border-top: 8px solid {};", color)
                                    }
                                }>
                                    <strong style="font-size: 1.3rem; display: block; margin-bottom: 5px;">"Room " {r_cloned.number.clone()}</strong>
                                    <span style="font-size: 0.8rem; color: #7f8c8d; background: #f8f9fa; padding: 2px 8px; border-radius: 10px;">{r_cloned.room_type.clone()} " • ₹" {r_cloned.price}</span>
                                    <div style={
                                        let rid = rid_fixed.clone();
                                        move || {
                                            let b_opt = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid);
                                            let color = if let Some(b) = b_opt {
                                                let p: f64 = b.payments.iter().map(|pay| pay.amount).sum();
                                                if b.total_amount > p { "#f39c12" } else { "#e74c3c" }
                                            } else { "#27ae60" };
                                            format!("margin: 15px 0; font-size: 0.8rem; font-weight: bold; color: {};", color)
                                        }
                                    }>
                                        {let rid = rid_fixed.clone(); move || {
                                            if let Some(b) = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid) {
                                                let p: f64 = b.payments.iter().map(|pay| pay.amount).sum();
                                                if b.total_amount > p { format!("● PENDING ₹{}", b.total_amount - p) } else { "● OCCUPIED".to_string() }
                                            } else { "● AVAILABLE".to_string() }
                                        }}
                                    </div>
                                    <div style="display: flex; gap: 8px; margin-top: 10px;">
                                        {let rb = r_cloned.clone(); let rid_b = rid_fixed.clone(); view! { <button on:click=move |_| set_show_book_modal.set(Some(rb.clone())) disabled={let rid=rid_b.clone(); move || get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid).is_some()} style="flex: 1; padding: 8px; font-size: 0.75rem; background: #27ae60;">"Book"</button> }}
                                        {let re = r_cloned.clone(); let rid_e = rid_fixed.clone(); view! { <button on:click=move |_| { if let Some(booking) = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid_e) { set_confirm_cancel_stay.set(false); set_confirm_checkout.set(false); set_show_manage_stay_modal.set(Some(booking)); } else { set_show_edit_room_modal.set(Some(re.clone())); } } style="flex: 1; padding: 8px; font-size: 0.75rem; background: #3498db;">"Edit"</button> }}
                                    </div>
                                </div>
                            }
                        } />
                    </div>
                }.into_view()
            }}

            <div style="margin-top: 3rem;">
                <h3 style="border-bottom: 2px solid var(--primary); padding-bottom: 10px; margin-bottom: 1rem;">"Guests on Day (" {move || selected_date.get()} ")"</h3>
                {move || if loading.get() { view! {}.into_view() } else {
                    view! {
                        <div style="overflow-x: auto;">
                            <table>
                                <thead><tr><th>"Guest(s)"</th><th>"Room"</th><th>"Status"</th><th>"Paid"</th><th>"Balance"</th><th>"Contact Info"</th><th>"Action"</th></tr></thead>
                                <tbody>
                                    <For each=move || guests_on_day() key=|item| item.0.id.clone().unwrap_or_default() children=move |(b, c, paid)| {
                                        let b_data = b.clone();
                                        let c_data = c.clone();
                                        let balance = b_data.total_amount - paid;
                                        let status = b_data.status.clone();
                                        view! {
                                            <tr>
                                                <td>
                                                    <strong>{b_data.customer_name.clone()}</strong><br/>
                                                    {b_data.extra_guests.iter().map(|g| view! { <div style="font-size: 0.8rem; color: #666;">"+ " {g.name.clone()}</div> }).collect_view()}
                                                </td>
                                                <td>"Room " {b_data.room_number.clone()}</td>
                                                <td><span style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.7rem; font-weight: bold; background: {}; color: white;", if status == "Checked-In" { "#27ae60" } else { "#95a5a6" })>{status.to_uppercase()}</span></td>
                                                <td style="color: #27ae60; font-weight: bold;">"₹" {paid}</td>
                                                <td style=format!("color: {}; font-weight: bold;", if balance > 0.0 { "#e67e22" } else { "#27ae60" })>"₹" {balance}</td>
                                                <td><small><strong>"Mob: "</strong> {c_data.as_ref().map(|x| x.phone.clone()).unwrap_or_default()}</small></td>
                                                <td><button on:click=move |_| set_show_bill_modal.set(Some((b_data.clone(), c_data.clone()))) style="padding: 5px 12px; font-size: 0.75rem; background: #8e44ad;">"Bill"</button></td>
                                            </tr>
                                        }
                                    }/>
                                </tbody>
                            </table>
                            {move || if guests_on_day().is_empty() { view! { <p style="text-align: center; color: #7f8c8d; padding: 2rem;">"No guest history for this day."</p> }.into_view() } else { view! {}.into_view() }}
                        </div>
                    }.into_view()
                }}
            </div>

            // --- QUICK BOOKING MODAL ---
            {move || show_book_modal.get().map(|room| {
                let (sel_cust, set_sel_cust) = create_signal("".to_string());
                let (extra_selected, set_extra_selected) = create_signal(Vec::<ExtraGuest>::new());
                let (final_price, set_final_price) = create_signal(room.price.to_string());
                let (paid_now, set_paid_now) = create_signal(room.price.to_string());
                let (check_out, set_check_out) = create_signal("".to_string());
                let (guest_search, set_guest_search) = create_signal("".to_string());
                let (saving, set_saving) = create_signal(false);
                let filtered = move || { let q = guest_search.get().to_lowercase(); customers.get().into_iter().filter(|c| c.full_name.to_lowercase().contains(&q) || c.aadhaar.contains(&q)).collect::<Vec<_>>() };
                let add_extra = move |_| {
                    let cid = sel_cust.get();
                    if cid.is_empty() { return; }
                    if let Some(c) = customers.get_untracked().into_iter().find(|cust| cust.id.as_deref() == Some(&cid)) {
                        if !extra_selected.get_untracked().iter().any(|g| g.id == cid) { set_extra_selected.update(|v| v.push(ExtraGuest { id: cid, name: c.full_name.clone() })); }
                    }
                };
                let r_cloned = room.clone();
                let handle_book = {
                    let r_cloned = r_cloned.clone();
                    move |ev: leptos::ev::SubmitEvent| {
                        ev.prevent_default(); set_saving.set(true);
                        let guests = extra_selected.get_untracked();
                        if guests.is_empty() { set_saving.set(false); return; }
                        let primary = &guests[0]; let extras = guests[1..].to_vec(); let date = selected_date.get_untracked();
                        let new_booking = NewBooking { room_id: r_cloned.id.clone().unwrap_or_default(), customer_id: primary.id.clone(), customer_name: primary.name.clone(), extra_guests: extras, room_number: r_cloned.number.clone(), check_in_date: date.clone(), check_out_date: check_out.get(), in_time: None, out_time: None, status: "Checked-In".to_string(), total_amount: final_price.get().parse::<f64>().unwrap_or(0.0), payments: vec![Payment { amount: paid_now.get().parse::<f64>().unwrap_or(0.0), date: date }] };
                        spawn_local(async move { wait_for_bridge().await; let _ = add_booking_js(serde_wasm_bindgen::to_value(&new_booking).unwrap()).await; set_show_book_modal.set(None); load_data(); });
                    }
                };
                let r_num_view = r_cloned.number.clone();
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;">
                        <div class="card" style="width: 100%; max-width: 450px; padding: 2rem;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h3>"Quick Check-in"</h3><button on:click=move |_| set_show_book_modal.set(None) style="background: none; color: black; font-size: 1.5rem;">"×"</button></div>
                            <form on:submit=handle_book>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div style="display: flex; gap: 10px;"><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Room"</label><input type="text" value=r_num_view disabled style="background: #eee;" /></div><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Check-in"</label><input type="text" value=selected_date.get() disabled style="background: #eee;" /></div></div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Select Guest(s)"</label>
                                        <div style="display: flex; gap: 5px; margin-bottom: 5px;">
                                            <input type="text" placeholder="Search..." on:input=move |ev| set_guest_search.set(event_target_value(&ev)) style="flex: 1;" />
                                            <button type="button" on:click=add_extra style="padding: 0 15px; background: #27ae60; font-weight: bold;">"+"</button>
                                            <button type="button" on:click=move |_| set_show_add_guest_modal.set(true) style="padding: 0 15px; background: #3498db; font-weight: bold;" title="Register New Guest">"New"</button>
                                        </div>
                                        <select on:change=move |ev| set_sel_cust.set(event_target_value(&ev)) prop:value=sel_cust>
                                            <option value="">"Choose guest from search..."</option>
                                            {move || filtered().into_iter().map(|c| { let cid = c.id.clone().unwrap_or_default(); view! { <option value=cid>{c.full_name.clone()} " (" {c.phone.clone()} ")" </option> } }).collect_view()}
                                        </select><div style="margin-top: 10px; display: flex; flex-wrap: wrap; gap: 5px;">{move || extra_selected.get().into_iter().enumerate().map(|(idx, g)| { let g_id_val = g.id.clone(); let g_name_val = g.name.clone(); view! { <span style="background: #3498db; color: white; padding: 2px 10px; border-radius: 15px; font-size: 0.8rem;">{if idx==0 { "(P) " } else { "" }} {g_name_val} <button type="button" on:click=move |_| set_extra_selected.update(|v| v.retain(|x| x.id != g_id_val)) style="background:none; padding:0; margin-left:5px; font-weight:bold; color:white;">"×"</button></span> } }).collect_view()}</div></div>
                                    <div style="display: flex; gap: 10px;"><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Total Price"</label><input type="number" on:input=move |ev| set_final_price.set(event_target_value(&ev)) prop:value=final_price required /></div><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Paying Now"</label><input type="number" on:input=move |ev| set_paid_now.set(event_target_value(&ev)) prop:value=paid_now required /></div></div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Estimated Check-out"</label><input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) required /></div>
                                </div>
                                <div style="display: flex; gap: 10px; margin-top: 25px;"><button type="submit" disabled=move || saving.get() || extra_selected.get().is_empty() style="flex: 2; background: #27ae60;">"Confirm"</button><button type="button" on:click=move |_| set_show_book_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button></div>
                            </form>
                        </div>
                    </div>
                }
            })}

            {move || if show_add_guest_modal.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 4000; padding: 1rem;"><div class="card" style="width: 100%; max-width: 500px; padding: 1rem; max-height: 90vh; overflow-y: auto;"><div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h3>"Register New Guest"</h3><button on:click=move |_| set_show_add_guest_modal.set(false) style="background: none; color: black; font-size: 1.5rem;">"×"</button></div><CustomerForm editing_id=create_memo(|_| None) initial_data=None on_success=Callback::new(move |_| { set_show_add_guest_modal.set(false); load_data(); }) /></div></div> }.into_view() } else { view! {}.into_view() }}

            {move || show_manage_stay_modal.get().map(|booking| {
                let (check_out, set_check_out) = create_signal(booking.check_out_date.clone());
                let (extra_payment, set_extra_payment) = create_signal((booking.total_amount - booking.payments.iter().map(|p| p.amount).sum::<f64>()).to_string());
                let (saving, set_saving) = create_signal(false);
                let b_data_orig = booking.clone();
                let b_id_val = b_data_orig.id.clone().unwrap_or_default();
                let b_id_val2 = b_id_val.clone();
                let handle_update = {
                    let b_data = b_data_orig.clone();
                    move |ev: leptos::ev::SubmitEvent| {
                        ev.prevent_default(); set_saving.set(true);
                        let mut upd_pays = b_data.payments.clone(); let extra = extra_payment.get().parse::<f64>().unwrap_or(0.0);
                        if extra > 0.0 { upd_pays.push(Payment { amount: extra, date: selected_date.get_untracked() }); }
                        let upd_book = NewBooking { room_id: b_data.room_id.clone(), customer_id: b_data.customer_id.clone(), customer_name: b_data.customer_name.clone(), extra_guests: b_data.extra_guests.clone(), room_number: b_data.room_number.clone(), check_in_date: b_data.check_in_date.clone(), check_out_date: check_out.get(), in_time: b_data.in_time.clone(), out_time: b_data.out_time.clone(), status: b_data.status.clone(), total_amount: b_data.total_amount, payments: upd_pays };
                        let bid = b_id_val2.clone();
                        spawn_local(async move { wait_for_bridge().await; let _ = update_booking_js(bid, serde_wasm_bindgen::to_value(&upd_book).unwrap()).await; set_show_manage_stay_modal.set(None); load_data(); });
                    }
                };
                let b_data_co = b_data_orig.clone();
                let b_id_co_val = b_data_co.id.clone().unwrap_or_default();
                let on_checkout_final = move |_| {
                    let today = selected_date.get_untracked(); let paid_so_far: f64 = b_data_co.payments.iter().map(|p| p.amount).sum();
                    let mut final_payments = b_data_co.payments.clone();
                    if b_data_co.total_amount > paid_so_far { final_payments.push(Payment { amount: b_data_co.total_amount - paid_so_far, date: today.clone() }); }
                    let co_booking = NewBooking { room_id: b_data_co.room_id.clone(), customer_id: b_data_co.customer_id.clone(), customer_name: b_data_co.customer_name.clone(), extra_guests: b_data_co.extra_guests.clone(), room_number: b_data_co.room_number.clone(), check_in_date: b_data_co.check_in_date.clone(), check_out_date: today, in_time: b_data_co.in_time.clone(), out_time: None, status: "Checked-Out".to_string(), total_amount: b_data_co.total_amount, payments: final_payments };
                    let bid = b_id_co_val.clone(); let rid = b_data_co.room_id.clone();
                    spawn_local(async move { wait_for_bridge().await; let _ = update_booking_js(bid, serde_wasm_bindgen::to_value(&co_booking).unwrap()).await; let _ = update_room_js(rid, JsValue::from_str("Available")).await; set_show_manage_stay_modal.set(None); load_data(); });
                };
                let b_id_del = b_data_orig.id.clone().unwrap_or_default(); let b_rid_del = b_data_orig.room_id.clone();
                let b_data_v = b_data_orig.clone();
                view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: center; justify-content: center; z-index: 3000; padding: 1rem; display: flex; align-items: center;"><div class="card" style="width: 100%; max-width: 450px; padding: 2rem;"><h3>"Manage Guest Stay"</h3><div style="margin-bottom: 20px; text-align: left; background: #f8f9fa; padding: 1rem; border-radius: 8px;"><p><strong>"Guest(s): "</strong> {b_data_v.customer_name.clone()} {b_data_v.extra_guests.iter().map(|g| format!(", {}", g.name)).collect::<String>()}</p><p><strong>"Room: "</strong> {b_data_v.room_number.clone()} " | " <strong>"In: "</strong> {b_data_v.check_in_date.clone()}</p></div><form on:submit=handle_update><div style="display: flex; flex-direction: column; gap: 15px; text-align: left;"><div><label style="font-size: 0.8rem; font-weight: bold;">"Collect Payment"</label><input type="number" on:input=move |ev| set_extra_payment.set(event_target_value(&ev)) prop:value=extra_payment /></div><div><label style="font-size: 0.8rem; font-weight: bold;">"Update Check-out"</label><input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) prop:value=check_out required /></div></div><div style="display: flex; flex-direction: column; gap: 10px; margin-top: 25px;"><button type="submit" disabled=saving style="background: #3498db;">"Save Changes"</button>{let co_f = on_checkout_final.clone(); move || if confirm_checkout.get() { let co_f2 = co_f.clone(); view! { <div style="background: #dcfce7; padding: 10px; border-radius: 8px; border: 1px solid #22c55e; margin-top: 10px;"><p style="color: #166534; font-weight: bold; margin-bottom: 10px;">"Finalize Checkout?"</p><div style="display: flex; gap: 5px;"><button type="button" on:click=move |_| co_f2(()) style="background: #22c55e; flex: 1;">"YES"</button><button type="button" on:click=move |_| set_confirm_checkout.set(false) style="background: #6c757d; flex: 1;">"No"</button></div></div> }.into_view() } else { view! { <button type="button" on:click=move |_| set_confirm_checkout.set(true) style="background: #27ae60;">"Proceed to Checkout"</button> }.into_view() }}{let bid_d = b_id_del.clone(); let rid_d = b_rid_del.clone(); move || if confirm_cancel_stay.get() { let bid_d2 = bid_d.clone(); let rid_d2 = rid_d.clone(); view! { <div style="background: #fee2e2; padding: 10px; border-radius: 8px; border: 1px solid #ef4444; margin-top: 10px;"><p style="color: #991b1b; font-weight: bold; margin-bottom: 10px;">"Permanently delete stay?"</p><div style="display: flex; gap: 5px;"><button type="button" on:click=move |_| { let bid = bid_d2.clone(); let rid = rid_d2.clone(); spawn_local(async move { wait_for_bridge().await; let _ = crate::api::delete_booking_js(bid, rid).await; set_show_manage_stay_modal.set(None); load_data(); }); } style="background: #ef4444; flex: 1;">"DELETE"</button><button type="button" on:click=move |_| set_confirm_cancel_stay.set(false) style="background: #6c757d; flex: 1;">"Cancel"</button></div></div> }.into_view() } else { view! { <button type="button" on:click=move |_| set_confirm_cancel_stay.set(true) style="background: #e74c3c;">"Delete/Cancel Stay"</button> }.into_view() }}<button type="button" on:click=move |_| set_show_manage_stay_modal.set(None) style="background: #6c757d; margin-top: 10px;">"Close"</button></div></form></div></div> }
            })}

            {move || show_edit_room_modal.get().map(|room| {
                let (r_type, set_r_type) = create_signal(room.room_type.clone());
                let (r_price, set_r_price) = create_signal(room.price.to_string());
                let (saving, set_saving) = create_signal(false);
                let r_id_val = room.id.clone().unwrap_or_default();
                let r_num = room.number.clone();
                let handle_room_update = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default(); set_saving.set(true);
                    let updated_room = NewRoom { number: r_num.clone(), room_type: r_type.get(), status: "Available".to_string(), price: r_price.get().parse().unwrap_or(0.0) };
                    let rid = r_id_val.clone();
                    spawn_local(async move { wait_for_bridge().await; let _ = update_room_js(rid, serde_wasm_bindgen::to_value(&updated_room).unwrap()).await; set_show_edit_room_modal.set(None); load_data(); });
                };
                let r_num_v = room.number.clone();
                view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;"><div class="card" style="width: 100%; max-width: 400px; padding: 2rem;"><h3>"Edit Room Settings"</h3><form on:submit=handle_room_update><div style="display: flex; flex-direction: column; gap: 15px; text-align: left;"><div><label style="font-size: 0.8rem; font-weight: bold;">"Room Number"</label><input type="text" value=r_num_v disabled style="background: #eee;" /></div><div><label style="font-size: 0.8rem; font-weight: bold;">"Category"</label><select on:change=move |ev| set_r_type.set(event_target_value(&ev)) prop:value=r_type><option value="Deluxe">"Deluxe"</option><option value="AC">"AC"</option><option value="non-AC">"non-AC"</option></select></div><div><label style="font-size: 0.8rem; font-weight: bold;">"Base Price"</label><input type="number" on:input=move |ev| set_r_price.set(event_target_value(&ev)) prop:value=r_price /></div></div><div style="display: flex; gap: 10px; margin-top: 25px;"><button type="submit" disabled=saving style="flex: 1; background: #3498db;">"Save"</button><button type="button" on:click=move |_| set_show_edit_room_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button></div></form></div></div> }
            })}
            </div>
        </div>
    }
}
