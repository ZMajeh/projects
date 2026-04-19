use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use crate::models::{User, Room, Booking, NewBooking, NewRoom, Customer, Payment, ExtraGuest};
use crate::utils::{clear_user, wait_for_bridge};
use crate::api::{sign_out_user, add_booking_js, update_room_js, update_booking_js, delete_booking_js};
use crate::components::rooms::{fetch_rooms, Rooms};
use crate::components::bookings::{fetch_bookings, Bookings};
use crate::components::customers::{fetch_customers, Customers, CustomerForm};

fn get_active_booking_helper(bookings: &[Booking], date_str: &str, room_id: &str) -> Option<Booking> {
    bookings.iter().find(|b| {
        b.room_id == room_id && 
        b.status == "Checked-In" &&
        date_str >= b.check_in_date.as_str() && 
        date_str < b.check_out_date.as_str()
    }).cloned()
}

#[component]
fn PrintableBill(booking: Booking, customer: Option<Customer>, on_close: Callback<()>) -> impl IntoView {
    let paid: f64 = booking.payments.iter().map(|p| p.amount).sum();
    let balance = booking.total_amount - paid;
    let today = js_sys::Date::new_0().to_iso_string().as_string().unwrap()[..10].to_string();
    
    view! {
        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: #f0f2f5; z-index: 5000; overflow-y: auto; padding: 2rem;" class="bill-page">
            // Header Controls
            <div style="max-width: 850px; margin: 0 auto 1rem auto; display: flex; justify-content: space-between; align-items: center;" class="no-print">
                <button on:click=move |_| on_close.call(()) style="background: #34495e; padding: 10px 20px;">"← Back to Dashboard"</button>
                <button on:click=move |_| { let _ = window().print(); } style="background: #27ae60; padding: 10px 30px; font-weight: bold; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">"PRINT INVOICE"</button>
            </div>

            // The Bill
            <div style="max-width: 850px; margin: 0 auto; background: white; padding: 40px; box-shadow: 0 10px 30px rgba(0,0,0,0.1); border: 2px solid #2c3e50; position: relative; min-height: 1000px;" class="bill-container">
                
                // Decorative Corner
                <div style="position: absolute; top: 0; right: 0; width: 150px; height: 150px; background: linear-gradient(135deg, transparent 50%, #2c3e50 50%); border-top-right-radius: 0;"></div>

                // Header Logic
                <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 40px; border-bottom: 4px solid #2c3e50; padding-bottom: 20px;">
                    <div>
                        <h1 style="margin: 0; font-size: 3rem; color: #2c3e50; text-transform: uppercase; letter-spacing: 5px; font-weight: 900;">"ANAND"</h1>
                        <h3 style="margin: 0; font-size: 1.2rem; color: #7f8c8d; letter-spacing: 8px; text-transform: uppercase;">"Lodge & Stay"</h3>
                        <div style="margin-top: 15px; font-size: 0.9rem; color: #34495e; line-height: 1.4;">
                            <p style="margin: 0;">"Main Road, Near Railway Station"</p>
                            <p style="margin: 0;">"Contact: +91 98XXX XXXXX • GSTIN: 27AAAAA0000A1Z5"</p>
                        </div>
                    </div>
                    <div style="text-align: right; margin-top: 10px;">
                        <div style="background: #2c3e50; color: white; padding: 10px 20px; display: inline-block; border-radius: 4px; margin-bottom: 10px;">
                            <h2 style="margin: 0; font-size: 1.2rem; letter-spacing: 2px;">"INVOICE"</h2>
                        </div>
                        <p style="margin: 0; font-size: 0.9rem;"><strong>"Date: "</strong> {today}</p>
                        <p style="margin: 0; font-size: 0.9rem;"><strong>"Bill No: "</strong> "AL-" {booking.id.clone().unwrap_or_default().chars().take(6).collect::<String>().to_uppercase()}</p>
                    </div>
                </div>

                // Details Grid
                <div style="display: grid; grid-template-columns: 1.2fr 1fr; gap: 30px; margin-bottom: 40px;">
                    <div style="background: #f8f9fa; padding: 20px; border-left: 5px solid #2c3e50; border-radius: 4px;">
                        <h4 style="margin: 0 0 15px 0; color: #2c3e50; border-bottom: 1px solid #ddd; padding-bottom: 5px; text-transform: uppercase; font-size: 0.85rem; letter-spacing: 1px;">"Billed To (Guest)"</h4>
                        <p style="margin: 5px 0; font-size: 1.1rem;"><strong>{booking.customer_name.clone()}</strong></p>
                        {if !booking.extra_guests.is_empty() {
                            view! {
                                <p style="margin: 5px 0; font-size: 0.85rem; color: #666;">
                                    "With: " {booking.extra_guests.iter().map(|g| g.name.clone()).collect::<Vec<_>>().join(", ")}
                                </p>
                            }.into_view()
                        } else { view! {}.into_view() }}
                        <p style="margin: 5px 0; font-size: 0.9rem;">"Mob: " {customer.as_ref().map(|c| c.phone.clone()).unwrap_or_else(|| "N/A".to_string())}</p>
                        <p style="margin: 5px 0; font-size: 0.9rem;">"Aadhar: " {customer.as_ref().map(|c| c.aadhaar.clone()).unwrap_or_else(|| "N/A".to_string())}</p>
                    </div>
                    <div style="background: #f8f9fa; padding: 20px; border-left: 5px solid #3498db; border-radius: 4px;">
                        <h4 style="margin: 0 0 15px 0; color: #2c3e50; border-bottom: 1px solid #ddd; padding-bottom: 5px; text-transform: uppercase; font-size: 0.85rem; letter-spacing: 1px;">"Stay Information"</h4>
                        <table style="width: 100%; font-size: 0.9rem;">
                            <tr><td style="padding: 3px 0; color: #7f8c8d;">"Room No:"</td><td style="text-align: right;"><strong>"Room " {booking.room_number.clone()}</strong></td></tr>
                            <tr><td style="padding: 3px 0; color: #7f8c8d;">"Arrival:"</td><td style="text-align: right;">{booking.check_in_date.clone()} " (" {booking.in_time.clone().unwrap_or_else(|| "--:--".to_string())} ")"</td></tr>
                            <tr><td style="padding: 3px 0; color: #7f8c8d;">"Departure:"</td><td style="text-align: right;">{booking.check_out_date.clone()} " (" {booking.out_time.clone().unwrap_or_else(|| "--:--".to_string())} ")"</td></tr>
                            <tr><td style="padding: 3px 0; color: #7f8c8d;">"Status:"</td><td style="text-align: right; color: #27ae60;"><strong>{booking.status.clone().to_uppercase()}</strong></td></tr>
                        </table>
                    </div>
                </div>

                // Charges Table
                <table style="width: 100%; border-collapse: collapse; margin-bottom: 40px; border: 1px solid #eee;">
                    <thead>
                        <tr style="background: #2c3e50; color: white;">
                            <th style="text-align: left; padding: 15px; text-transform: uppercase; font-size: 0.8rem; letter-spacing: 1px;">"Description"</th>
                            <th style="text-align: center; padding: 15px; text-transform: uppercase; font-size: 0.8rem; letter-spacing: 1px;">"Price/Day"</th>
                            <th style="text-align: center; padding: 15px; text-transform: uppercase; font-size: 0.8rem; letter-spacing: 1px;">"Qty"</th>
                            <th style="text-align: right; padding: 15px; text-transform: uppercase; font-size: 0.8rem; letter-spacing: 1px;">"Total"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr style="border-bottom: 1px solid #eee;">
                            <td style="padding: 20px; font-weight: bold;">"Room Accommodation Charges" <br/><small style="font-weight: normal; color: #7f8c8d;">"Delux Category Room stay"</small></td>
                            <td style="text-align: center; padding: 20px;">"₹" {booking.total_amount}</td>
                            <td style="text-align: center; padding: 20px;">"1"</td>
                            <td style="text-align: right; padding: 20px; font-weight: bold;">"₹" {booking.total_amount}</td>
                        </tr>
                    </tbody>
                </table>

                // Summary and Payments
                <div style="display: grid; grid-template-columns: 1fr 300px; gap: 40px;">
                    <div>
                        <h4 style="margin: 0 0 10px 0; font-size: 0.8rem; text-transform: uppercase; color: #7f8c8d;">"Payment History"</h4>
                        <table style="width: 100%; font-size: 0.8rem; border-collapse: collapse;">
                            {booking.payments.iter().map(|p| view! {
                                <tr style="border-bottom: 1px solid #f0f0f0;">
                                    <td style="padding: 5px 0;">{p.date.clone()}</td>
                                    <td style="padding: 5px 0; color: #7f8c8d;">"Via Cash/Online"</td>
                                    <td style="padding: 5px 0; text-align: right; color: #27ae60;">"₹" {p.amount}</td>
                                </tr>
                            }).collect_view()}
                        </table>
                    </div>
                    <div>
                        <div style="background: #f8f9fa; padding: 20px; border-radius: 4px;">
                            <div style="display: flex; justify-content: space-between; margin-bottom: 10px;">
                                <span>"Sub Total:"</span>
                                <span>"₹" {booking.total_amount}</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; margin-bottom: 10px; color: #27ae60;">
                                <span>"Total Paid:"</span>
                                <span>"- ₹" {paid}</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; border-top: 2px solid #2c3e50; padding-top: 10px; font-size: 1.3rem; font-weight: 900; color: #2c3e50;">
                                <span>"DUE:"</span>
                                <span>"₹" {balance}</span>
                            </div>
                        </div>
                    </div>
                </div>

                // Signatures
                <div style="margin-top: 80px; display: flex; justify-content: space-between; align-items: flex-end;">
                    <div style="text-align: center;">
                        <p style="font-size: 0.8rem; color: #7f8c8d; margin-bottom: 50px;">"For Guest Signature"</p>
                        <div style="width: 180px; border-bottom: 1px dashed #333;"></div>
                    </div>
                    <div style="text-align: center;">
                        <div style="margin-bottom: 10px; color: #2c3e50; font-weight: bold; font-family: 'Brush Script MT', cursive; font-size: 1.5rem;">"Anand"</div>
                        <p style="font-size: 0.8rem; color: #7f8c8d; margin-bottom: 10px;">"Authorized Signatory"</p>
                        <div style="width: 180px; border-bottom: 1px solid #333;"></div>
                    </div>
                </div>

                // Legal Footer
                <div style="margin-top: 60px; border-top: 1px solid #eee; padding-top: 20px; font-size: 0.75rem; color: #95a5a6; line-height: 1.6;">
                    <p style="margin: 0; font-weight: bold; color: #7f8c8d; margin-bottom: 5px;">"Terms & Conditions:"</p>
                    <ol style="margin: 0; padding-left: 15px;">
                        <li>"Checkout time is 12:00 Noon. Late checkout may incur additional charges."</li>
                        <li>"Guests are requested to produce original ID at the time of check-in."</li>
                        <li>"The management is not responsible for loss of any valuables."</li>
                        <li>"Any damage to room property will be charged to the guest."</li>
                    </ol>
                </div>
            </div>

            <style>
                "@media print { 
                    .no-print { display: none !important; } 
                    body { background: white !important; padding: 0 !important; } 
                    .bill-page { padding: 0 !important; background: white !important; position: static !important; }
                    .bill-container { box-shadow: none !important; border: 1px solid #eee !important; width: 100% !important; max-width: none !important; margin: 0 !important; }
                }"
            </style>
        </div>
    }
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
        let current_ms = js_sys::Date::parse(&selected_date.get_untracked());
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
            {move || show_bill_modal.get().map(|(b, c)| view! { <PrintableBill booking=b customer=c on_close=Callback::new(move |_| set_show_bill_modal.set(None)) /> })}

            <div style="display: flex; flex-direction: column; align-items: center; gap: 1rem; margin-bottom: 2rem;">
                <h2 style="color: var(--primary); font-size: 1.8rem; font-weight: 800;">"Anand Lodge Occupancy"</h2>
                <div style="display: flex; gap: 20px; align-items: center; width: 100%; justify_content: center; flex-wrap: wrap;">
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
                                    <strong style="font-size: 1.3rem; display: block; margin-bottom: 5px;">"Room " {r_cloned.number.clone()}</strong>
                                    <span style="font-size: 0.8rem; color: #7f8c8d; background: #f8f9fa; padding: 2px 8px; border-radius: 10px;">{r_cloned.room_type.clone()} " • ₹" {r_cloned.price}</span>
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
                                        {let re = r_cloned.clone(); view! { <button on:click=move |_| { if let Some(booking) = get_active_booking_helper(&bookings.get(), &selected_date.get(), &rid_edit) { set_confirm_cancel_stay.set(false); set_confirm_checkout.set(false); set_show_manage_stay_modal.set(Some(booking)); } else { set_show_edit_room_modal.set(Some(re.clone())); } } style="flex: 1; padding: 8px; font-size: 0.75rem; background: #3498db;">"Edit"</button> }}
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
                                <thead>
                                    <tr>
                                        <th>"Guest(s)"</th>
                                        <th>"Room"</th>
                                        <th>"Status"</th>
                                        <th>"Paid"</th>
                                        <th>"Balance"</th>
                                        <th>"Contact Info"</th>
                                        <th>"Action"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For each=move || guests_on_day() key=|item| item.0.id.clone().unwrap_or_default() children=move |(b, c, paid)| {
                                        let balance = b.total_amount - paid;
                                        let c_name = b.customer_name.clone();
                                        let extra_guests = b.extra_guests.clone();
                                        let status = b.status.clone();
                                        let age = c.as_ref().and_then(|x| x.age.clone()).unwrap_or_else(|| "??".to_string());
                                        let phone = c.as_ref().map(|x| x.phone.clone()).unwrap_or_else(|| "N/A".to_string());
                                        let aadhaar = c.as_ref().map(|x| x.aadhaar.clone()).unwrap_or_else(|| "N/A".to_string());
                                        let b_bill = b.clone();
                                        let c_bill = c.clone();
                                        view! {
                                            <tr>
                                                <td>
                                                    <strong>{c_name}</strong><br/>
                                                    {extra_guests.into_iter().map(|g| view! { <div style="font-size: 0.8rem; color: #666;">"+ " {g.name}</div> }).collect_view()}
                                                    <small>{age} " yrs"</small>
                                                </td>
                                                <td>"Room " {b.room_number.clone()}</td>
                                                <td><span style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.7rem; font-weight: bold; background: {}; color: white;", if status == "Checked-In" { "#27ae60" } else { "#95a5a6" })>{status.to_uppercase()}</span></td>
                                                <td style="color: #27ae60; font-weight: bold;">"₹" {paid}</td>
                                                <td style=format!("color: {}; font-weight: bold;", if balance > 0.0 { "#e67e22" } else { "#27ae60" })>"₹" {balance}</td>
                                                <td><small><strong>"Mob: "</strong> {phone}</small><br/><small><strong>"Aadhar: "</strong> {aadhaar}</small></td>
                                                <td><button on:click=move |_| set_show_bill_modal.set(Some((b_bill.clone(), c_bill.clone()))) style="padding: 5px 12px; font-size: 0.75rem; background: #8e44ad;">"Bill"</button></td>
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
                
                let filtered_guests = move || { let q = guest_search.get().to_lowercase(); customers.get().into_iter().filter(|c| c.full_name.to_lowercase().contains(&q) || c.aadhaar.contains(&q)).collect::<Vec<_>>() };
                
                let add_extra_guest = move |_| {
                    let cid = sel_cust.get();
                    if cid.is_empty() { return; }
                    if let Some(c) = customers.get_untracked().into_iter().find(|c| c.id.as_deref() == Some(&cid)) {
                        let already_in = extra_selected.get_untracked().iter().any(|g| g.id == cid);
                        if !already_in {
                            set_extra_selected.update(|v| v.push(ExtraGuest { id: cid, name: c.full_name }));
                        }
                    }
                };

                let r_id = room.id.clone().unwrap_or_default();
                let r_num = room.number.clone();
                let handle_book = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default(); set_saving.set(true);
                    let guests = extra_selected.get_untracked();
                    if guests.is_empty() { set_saving.set(false); return; }
                    let primary = &guests[0];
                    let extras = guests[1..].to_vec();
                    let date = selected_date.get_untracked();
                    let cout = check_out.get();
                    let total = final_price.get().parse::<f64>().unwrap_or(0.0);
                    let paid = paid_now.get().parse::<f64>().unwrap_or(0.0);
                    
                    let new_booking = NewBooking { 
                        room_id: r_id.clone(), 
                        customer_id: primary.id.clone(), 
                        customer_name: primary.name.clone(), 
                        extra_guests: extras,
                        room_number: r_num.clone(), 
                        check_in_date: date.clone(), 
                        check_out_date: cout, 
                        in_time: None, out_time: None, status: "Checked-In".to_string(), 
                        total_amount: total, 
                        payments: vec![Payment { amount: paid, date: date }] 
                    };
                    spawn_local(async move { wait_for_bridge().await; let _ = add_booking_js(serde_wasm_bindgen::to_value(&new_booking).unwrap()).await; set_show_book_modal.set(None); load_data(); });
                };

                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;">
                        <div class="card" style="width: 100%; max-width: 450px; padding: 2rem;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h3>"Quick Check-in"</h3><button on:click=move |_| set_show_add_guest_modal.set(true) style="font-size: 0.7rem; background: #e67e22; padding: 5px 10px;">"+ New Guest"</button></div>
                            <form on:submit=handle_book>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div style="display: flex; gap: 10px;"><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Room"</label><input type="text" value=room.number.clone() disabled style="background: #eee;" /></div><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Check-in"</label><input type="text" value=selected_date.get() disabled style="background: #eee;" /></div></div>
                                    
                                    <div>
                                        <label style="font-size: 0.8rem; font-weight: bold;">"Select Guest(s)"</label>
                                        <div style="display: flex; gap: 5px; margin-bottom: 5px;">
                                            <input type="text" placeholder="Search..." on:input=move |ev| set_guest_search.set(event_target_value(&ev)) style="flex: 1;" />
                                            <button type="button" on:click=add_extra_guest style="padding: 0 15px; background: #27ae60; font-weight: bold;">"+"</button>
                                        </div>
                                        <select on:change=move |ev| set_sel_cust.set(event_target_value(&ev)) prop:value=sel_cust>
                                            <option value="">"Choose guest from search..."</option>
                                            {move || filtered_guests().into_iter().map(|c| { let cid = c.id.clone().unwrap_or_default(); view! { <option value=cid>{c.full_name.clone()} " (" {c.phone.clone()} ")" </option> } }).collect_view()}
                                        </select>
                                        <div style="margin-top: 10px; display: flex; flex-wrap: wrap; gap: 5px;">
                                            {move || extra_selected.get().into_iter().enumerate().map(|(idx, g)| {
                                                let g_id = g.id.clone();
                                                view! { <span style="background: #3498db; color: white; padding: 2px 10px; border-radius: 15px; font-size: 0.8rem;">{if idx==0 { "(P) " } else { "" }} {g.name} <button type="button" on:click=move |_| set_extra_selected.update(|v| v.retain(|x| x.id != g_id)) style="background:none; padding:0; margin-left:5px; font-weight:bold;">"×"</button></span> }
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    <div style="display: flex; gap: 10px;"><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Total Price"</label><input type="number" on:input=move |ev| set_final_price.set(event_target_value(&ev)) prop:value=final_price required /></div><div style="flex: 1;"><label style="font-size: 0.8rem; font-weight: bold;">"Paying Now"</label><input type="number" on:input=move |ev| set_paid_now.set(event_target_value(&ev)) prop:value=paid_now required /></div></div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Estimated Check-out"</label><input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) required /></div>
                                </div>
                                <div style="display: flex; gap: 10px; margin-top: 25px;"><button type="submit" disabled=move || saving.get() || extra_selected.get().is_empty() style="flex: 2; background: #27ae60;">"Confirm"</button><button type="button" on:click=move |_| set_show_book_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button></div>
                            </form>
                        </div>
                    </div>
                }
            })}

            // --- ADD GUEST POPUP ---
            {move || if show_add_guest_modal.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 4000; padding: 1rem;"><div class="card" style="width: 100%; max-width: 500px; padding: 1rem; max-height: 90vh; overflow-y: auto;"><div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h3>"Register New Guest"</h3><button on:click=move |_| set_show_add_guest_modal.set(false) style="background: none; color: black; font-size: 1.5rem;">"×"</button></div><CustomerForm editing_id=create_memo(|_| None) initial_data=None on_success=Callback::new(move |_| { set_show_add_guest_modal.set(false); load_data(); }) /></div></div> }.into_view() } else { view! {}.into_view() }}

            // --- MANAGE STAY MODAL ---
            {move || show_manage_stay_modal.get().map(|booking| {
                let (check_out, set_check_out) = create_signal(booking.check_out_date.clone());
                let (extra_payment, set_extra_payment) = create_signal((booking.total_amount - booking.payments.iter().map(|p| p.amount).sum::<f64>()).to_string());
                let (saving, set_saving) = create_signal(false);
                let b_id = booking.id.clone().unwrap_or_default();
                let b_data_orig = booking.clone();
                let date_str = selected_date.get_untracked();
                
                let b_id_upd = b_id.clone(); let b_data_upd = b_data_orig.clone(); let date_upd = date_str.clone();
                let handle_update = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default(); set_saving.set(true);
                    let bid = b_id_upd.clone(); let extra = extra_payment.get().parse::<f64>().unwrap_or(0.0);
                    let mut updated_payments = b_data_upd.payments.clone();
                    if extra > 0.0 { updated_payments.push(Payment { amount: extra, date: date_upd.clone() }); }
                    let updated_booking = NewBooking { room_id: b_data_upd.room_id.clone(), customer_id: b_data_upd.customer_id.clone(), customer_name: b_data_upd.customer_name.clone(), extra_guests: b_data_upd.extra_guests.clone(), room_number: b_data_upd.room_number.clone(), check_in_date: b_data_upd.check_in_date.clone(), check_out_date: check_out.get(), in_time: b_data_upd.in_time.clone(), out_time: b_data_upd.out_time.clone(), status: b_data_upd.status.clone(), total_amount: b_data_upd.total_amount, payments: updated_payments };
                    spawn_local(async move { wait_for_bridge().await; let _ = update_booking_js(bid, serde_wasm_bindgen::to_value(&updated_booking).unwrap()).await; set_show_manage_stay_modal.set(None); load_data(); });
                };

                let b_id_co = b_id.clone(); let b_data_co = b_data_orig.clone(); let date_co = date_str.clone();
                let on_checkout_final = move |_| {
                    let bid = b_id_co.clone(); let b_data = b_data_co.clone(); let today = date_co.clone();
                    let paid_so_far: f64 = b_data.payments.iter().map(|p| p.amount).sum();
                    let mut final_payments = b_data.payments.clone();
                    if b_data.total_amount > paid_so_far { final_payments.push(Payment { amount: b_data.total_amount - paid_so_far, date: today.clone() }); }
                    let co_booking = NewBooking { room_id: b_data.room_id.clone(), customer_id: b_data.customer_id.clone(), customer_name: b_data.customer_name.clone(), extra_guests: b_data.extra_guests.clone(), room_number: b_data.room_number.clone(), check_in_date: b_data.check_in_date.clone(), check_out_date: today, in_time: b_data.in_time.clone(), out_time: None, status: "Checked-Out".to_string(), total_amount: b_data.total_amount, payments: final_payments };
                    spawn_local(async move { wait_for_bridge().await; let _ = update_booking_js(bid, serde_wasm_bindgen::to_value(&co_booking).unwrap()).await; let _ = update_room_js(b_data.room_id.clone(), JsValue::from_str("Available")).await; set_show_manage_stay_modal.set(None); load_data(); });
                };

                let b_id_del = b_id.clone(); let b_rid_del = b_data_orig.room_id.clone();
                let b_name_view = b_data_orig.customer_name.clone();
                let b_extras_view = b_data_orig.extra_guests.clone();
                let b_rnum_view = b_data_orig.room_number.clone();
                let b_checkin_view = b_data_orig.check_in_date.clone();

                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: center; justify-content: center; z-index: 3000; padding: 1rem; display: flex; align-items: center;">
                        <div class="card" style="width: 100%; max-width: 450px; padding: 2rem;">
                            <h3>"Manage Guest Stay"</h3>
                            <div style="margin-bottom: 20px; text-align: left; background: #f8f9fa; padding: 1rem; border-radius: 8px;"><p><strong>"Guest(s): "</strong> {b_name_view} {b_extras_view.iter().map(|g| format!(", {}", g.name)).collect::<String>()}</p><p><strong>"Room: "</strong> {b_rnum_view} " | " <strong>"In: "</strong> {b_checkin_view}</p></div>
                            <form on:submit=handle_update>
                                <div style="display: flex; flex-direction: column; gap: 15px; text-align: left;">
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Collect Payment"</label><input type="number" on:input=move |ev| set_extra_payment.set(event_target_value(&ev)) prop:value=extra_payment /></div>
                                    <div><label style="font-size: 0.8rem; font-weight: bold;">"Update Check-out"</label><input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) prop:value=check_out required /></div>
                                </div>
                                <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 25px;">
                                    <button type="submit" disabled=saving style="background: #3498db;">"Save Changes"</button>
                                    {let co_f = on_checkout_final.clone(); move || if confirm_checkout.get() { let co_f2 = co_f.clone(); view! { <div style="background: #dcfce7; padding: 10px; border-radius: 8px; border: 1px solid #22c55e; margin-top: 10px;"><p style="color: #166534; font-weight: bold; margin-bottom: 10px;">"Finalize Checkout?"</p><div style="display: flex; gap: 5px;"><button type="button" on:click=move |_| co_f2(()) style="background: #22c55e; flex: 1;">"YES"</button><button type="button" on:click=move |_| set_confirm_checkout.set(false) style="background: #6c757d; flex: 1;">"No"</button></div></div> }.into_view() } else { view! { <button type="button" on:click=move |_| { set_confirm_checkout.set(true); set_confirm_cancel_stay.set(false); } style="background: #27ae60; margin-top: 10px;">"Checkout (Paid & Completed)"</button> }.into_view() }}
                                    {move || if confirm_cancel_stay.get() { let bid=b_id_del.clone(); let rid=b_rid_del.clone(); view! { <div style="background: #fee2e2; padding: 10px; border-radius: 8px; border: 1px solid #ef4444; margin-top: 10px;"><p style="color: #b91c1c; font-weight: bold; margin-bottom: 10px;">"Really cancel?"</p><div style="display: flex; gap: 5px;"><button type="button" on:click=move |_| { let b=bid.clone(); let r=rid.clone(); spawn_local(async move { wait_for_bridge().await; let _ = delete_booking_js(b, r).await; set_show_manage_stay_modal.set(None); load_data(); }); } style="background: #ef4444; flex: 1;">"YES"</button><button type="button" on:click=move |_| set_confirm_cancel_stay.set(false) style="background: #6c757d; flex: 1;">"No"</button></div></div> }.into_view() } else { view! { <button type="button" on:click=move |_| { set_confirm_cancel_stay.set(true); set_confirm_checkout.set(false); } style="background: #e74c3c; margin-top: 10px;">"Cancel Stay (Delete)"</button> }.into_view() }}
                                    <button type="button" on:click=move |_| set_show_manage_stay_modal.set(None) style="background: #6c757d; margin-top: 10px;">"Close"</button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            })}

            // --- QUICK ROOM SETTINGS MODAL ---
            {move || show_edit_room_modal.get().map(|room| {
                let (r_type, set_r_type) = create_signal(room.room_type.clone());
                let (r_price, set_r_price) = create_signal(room.price.to_string());
                let (saving, set_saving) = create_signal(false);
                let r_id = room.id.clone().unwrap_or_default(); let r_num = room.number.clone();
                let handle_room_update = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default(); set_saving.set(true);
                    let rid = r_id.clone(); let rnum = r_num.clone();
                    let updated_room = NewRoom { number: rnum, room_type: r_type.get(), status: "Available".to_string(), price: r_price.get().parse().unwrap_or(0.0) };
                    spawn_local(async move { wait_for_bridge().await; let _ = update_room_js(rid, serde_wasm_bindgen::to_value(&updated_room).unwrap()).await; set_show_edit_room_modal.set(None); load_data(); });
                };
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; padding: 1rem;"><div class="card" style="width: 100%; max-width: 400px; padding: 2rem;"><h3>"Edit Room Settings"</h3><form on:submit=handle_room_update><div style="display: flex; flex-direction: column; gap: 15px; text-align: left;"><div><label style="font-size: 0.8rem; font-weight: bold;">"Room Number"</label><input type="text" value=room.number.clone() disabled style="background: #eee;" /></div><div><label style="font-size: 0.8rem; font-weight: bold;">"Category"</label><select on:change=move |ev| set_r_type.set(event_target_value(&ev)) prop:value=r_type><option value="Delux">"Delux"</option><option value="AC">"AC"</option><option value="non-AC">"non-AC"</option></select></div><div><label style="font-size: 0.8rem; font-weight: bold;">"Base Price"</label><input type="number" on:input=move |ev| set_r_price.set(event_target_value(&ev)) prop:value=r_price /></div></div><div style="display: flex; gap: 10px; margin-top: 25px;"><button type="submit" disabled=saving style="flex: 1; background: #3498db;">"Save"</button><button type="button" on:click=move |_| set_show_edit_room_modal.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button></div></form></div></div>
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
                <div style="padding: 1.5rem; border-bottom: 2px solid var(--primary); margin-bottom: 1rem; background: #f8f9fa;"><h1 style="color: var(--primary); font-size: 1.8rem; font-weight: 900; margin: 0; text-transform: uppercase; letter-spacing: 2px; text-shadow: 1px 1px 2px rgba(0,0,0,0.1);">"Anand"</h1><h2 style="color: #34495e; font-size: 1.1rem; font-weight: 700; margin: 0; opacity: 0.8;">"Lodge Manager"</h2></div>
                <A href="" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active" exact=true>"Overview"</A>
                <A href="rooms" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Rooms"</A>
                <A href="customers" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Customers"</A>
                <A href="bookings" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Bookings"</A>
                <div style="margin-top: auto; padding: 1rem; border-top: 1px solid #ddd; font-size: 0.85rem;"><p style="color: #7f8c8d; overflow: hidden; text-overflow: ellipsis; margin-bottom: 8px; padding-left: 1rem;">{user.email}</p><button on:click=handle_logout style="background-color: #e74c3c; width: calc(100% - 2rem); margin: 0 1rem; border-radius: 6px;">"Logout"</button></div>
            </nav>
            <main class="content">
                <header class="mobile-header"><button on:click=move |_| set_menu_open.update(|v| *v = !*v) style="background: none; color: black; font-size: 1.5rem; padding: 0;">"☰"</button><strong style="color: var(--primary); font-weight: 900; font-size: 1.2rem; letter-spacing: 1px;">"ANAND LODGE"</strong><div style="width: 30px;"></div></header>
                <div style="padding: 1rem;">{children()}</div>
            </main>
        </div> 
    }
}
