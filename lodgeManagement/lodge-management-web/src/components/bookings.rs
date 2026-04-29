use leptos::*;
use crate::models::{Booking, Customer, Payment, NewBooking};
use crate::api::{get_bookings_js, delete_booking_js, update_booking_js, update_room_js};
use crate::utils::wait_for_bridge;
use crate::components::dashboard::PrintableBill;
use crate::components::customers::fetch_customers;

pub async fn fetch_bookings() -> Vec<Booking> {
    wait_for_bridge().await;
    match get_bookings_js().await {
        Ok(js_val) => serde_wasm_bindgen::from_value(js_val).unwrap_or_default(),
        Err(_) => vec![],
    }
}

#[component]
pub fn Bookings() -> impl IntoView {
    let (bookings, set_bookings) = create_signal(Vec::<Booking>::new());
    let (loading, set_loading) = create_signal(true);
    let (confirm_delete_id, set_confirm_delete_id) = create_signal(None::<String>);
    let (show_bill_modal, set_show_bill_modal) = create_signal(None::<(Booking, Option<Customer>)>);
    let (confirm_checkout_booking, set_confirm_checkout_booking) = create_signal(None::<Booking>);

    let load_bookings = move || { spawn_local(async move { set_loading.set(true); set_bookings.set(fetch_bookings().await); set_loading.set(false); }); };
    create_effect(move |_| { load_bookings(); });

    let on_delete_final = move |id: String, room_id: String| {
        spawn_local(async move {
            wait_for_bridge().await;
            let _ = delete_booking_js(id, room_id).await;
            load_bookings();
        });
    };

    let on_checkout_final = move |b: Booking| {
        let b_cloned = b.clone();
        spawn_local(async move {
            wait_for_bridge().await;
            let today = js_sys::Date::new_0().to_iso_string().as_string().unwrap()[..10].to_string();
            let paid_so_far: f64 = b_cloned.payments.iter().map(|p| p.amount).sum();
            let mut final_payments = b_cloned.payments.clone();
            
            if b_cloned.total_amount > paid_so_far {
                final_payments.push(Payment { amount: b_cloned.total_amount - paid_so_far, date: today.clone() });
            }

            let co_booking = NewBooking {
                room_id: b_cloned.room_id.clone(),
                customer_id: b_cloned.customer_id.clone(),
                customer_name: b_cloned.customer_name.clone(),
                extra_guests: b_cloned.extra_guests.clone(),
                room_number: b_cloned.room_number.clone(),
                check_in_date: b_cloned.check_in_date.clone(),
                check_out_date: today,
                in_time: b_cloned.in_time.clone(),
                out_time: None,
                status: "Checked-Out".to_string(),
                total_amount: b_cloned.total_amount,
                payments: final_payments,
            };

            let bid = b_cloned.id.clone().unwrap_or_default();
            let rid = b_cloned.room_id.clone();
            let _ = update_booking_js(bid.clone(), serde_wasm_bindgen::to_value(&co_booking).unwrap()).await;
            let _ = update_room_js(rid, wasm_bindgen::JsValue::from_str("Available")).await;
            
            // Prepare data for the bill modal
            let final_b = Booking {
                id: Some(bid),
                room_id: co_booking.room_id,
                customer_id: co_booking.customer_id,
                customer_name: co_booking.customer_name,
                extra_guests: co_booking.extra_guests,
                room_number: co_booking.room_number,
                check_in_date: co_booking.check_in_date,
                check_out_date: co_booking.check_out_date,
                in_time: co_booking.in_time,
                out_time: co_booking.out_time,
                status: co_booking.status,
                total_amount: co_booking.total_amount,
                payments: co_booking.payments,
            };

            let all_custs = fetch_customers("".to_string()).await;
            let primary_cust = all_custs.into_iter().find(|c| c.id.as_deref() == Some(&final_b.customer_id));
            
            set_confirm_checkout_booking.set(None);
            load_bookings();
            set_show_bill_modal.set(Some((final_b, primary_cust)));
        });
    };

    view! {
        <div class="card">
            {move || show_bill_modal.get().map(|(b, c)| {
                let on_close = Callback::new(move |_| set_show_bill_modal.set(None));
                view! { <PrintableBill booking=b customer=c on_close=on_close /> }
            })}

            {move || confirm_checkout_booking.get().map(|b| {
                let b_val = b.clone();
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000;">
                        <div class="card" style="width: 100%; max-width: 400px; padding: 2rem; text-align: center;">
                            <h3 style="color: var(--primary);">"Finalize Checkout?"</h3>
                            <p>"This will mark the stay as Checked-Out and generate the bill."</p>
                            <p><strong>"Guest: "</strong> {b_val.customer_name.clone()}</p>
                            <div style="display: flex; gap: 10px; margin-top: 2rem;">
                                <button on:click=move |_| on_checkout_final(b_val.clone()) style="flex: 1; background: #27ae60;">"Confirm"</button>
                                <button on:click=move |_| set_confirm_checkout_booking.set(None) style="flex: 1; background: #6c757d;">"Cancel"</button>
                            </div>
                        </div>
                    </div>
                }
            })}

            <h1>"Recent Stays"</h1>
            {move || if loading.get() { view! { <p>"Loading stays..."</p> }.into_view() } else {
                view! {
                    <table>
                        <thead>
                            <tr style="background-color: #f2f2f2; text-align: left;">
                                <th>"Guest"</th>
                                <th>"Room"</th>
                                <th>"Dates"</th>
                                <th>"Status"</th>
                                <th>"Finance"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For each=move || bookings.get() key=|b| b.id.clone().unwrap_or_default() children=move |b| {
                                let id_c = b.id.clone().unwrap_or_default();
                                let rid_c = b.room_id.clone();
                                let total = b.total_amount;
                                let paid: f64 = b.payments.iter().map(|p| p.amount).sum();
                                let status = b.status.clone();
                                let b_cloned = b.clone();
                                
                                view! {
                                    <tr>
                                        <td>{b.customer_name.clone()}</td>
                                        <td>"Room " {b.room_number.clone()}</td>
                                        <td><small>{b.check_in_date.clone()} " to " {b.check_out_date.clone()}</small></td>
                                        <td>
                                            <span 
                                                on:click={
                                                    let b_click = b_cloned.clone();
                                                    move |_| {
                                                        if b_click.status == "Checked-In" {
                                                            set_confirm_checkout_booking.set(Some(b_click.clone()));
                                                        } else {
                                                            let b_bill = b_click.clone();
                                                            spawn_local(async move {
                                                                let all_custs = fetch_customers("".to_string()).await;
                                                                let primary_cust = all_custs.into_iter().find(|c| c.id.as_deref() == Some(&b_bill.customer_id));
                                                                set_show_bill_modal.set(Some((b_bill, primary_cust)));
                                                            });
                                                        }
                                                    }
                                                }
                                                style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.8rem; background-color: {}; color: white; cursor: pointer;", if status == "Checked-In" { "#27ae60" } else { "#7f8c8d" })
                                            >
                                                {status}
                                            </span>
                                        </td>
                                        <td>
                                            <div style="font-size: 0.8rem;">
                                                "Total: ₹" {total} <br/>
                                                <span style="color: #27ae60;">"Paid: ₹" {paid}</span>
                                            </div>
                                        </td>
                                        <td>
                                            {move || if confirm_delete_id.get() == Some(id_c.clone()) {
                                                let id_final = id_c.clone();
                                                let rid_final = rid_c.clone();
                                                view! {
                                                    <div style="display: flex; gap: 5px; align-items: center;">
                                                        <span style="font-size: 0.7rem; color: red;">"Sure?"</span>
                                                        <button on:click=move |_| on_delete_final(id_final.clone(), rid_final.clone()) style="padding: 2px 8px; font-size: 0.7rem; background: #e74c3c;">"YES"</button>
                                                        <button on:click=move |_| set_confirm_delete_id.set(None) style="padding: 2px 8px; font-size: 0.7rem; background: #6c757d;">"NO"</button>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                let id_del = id_c.clone();
                                                view! { <button on:click=move |_| set_confirm_delete_id.set(Some(id_del.clone())) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Delete"</button> }.into_view()
                                            }}
                                        </td>
                                    </tr>
                                }
                            } />
                        </tbody>
                    </table>
                }.into_view()
            }}
        </div>
    }
}
