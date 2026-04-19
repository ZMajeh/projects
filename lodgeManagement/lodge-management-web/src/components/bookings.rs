use leptos::*;
use crate::models::{Booking, NewBooking, Room, Customer};
use crate::api::{get_bookings_js, add_booking_js};
use crate::utils::wait_for_bridge;
use crate::components::rooms::fetch_rooms;
use crate::components::customers::fetch_customers;

pub async fn fetch_bookings() -> Vec<Booking> {
    wait_for_bridge().await;
    match get_bookings_js().await {
        Ok(js_val) => {
            match serde_wasm_bindgen::from_value::<Vec<Booking>>(js_val) {
                Ok(bookings) => {
                    logging::log!("RUST: Successfully deserialized {} bookings", bookings.length());
                    bookings
                },
                Err(e) => {
                    logging::error!("RUST ERROR: Failed to deserialize bookings: {:?}", e);
                    vec![]
                }
            }
        },
        Err(_) => vec![],
    }
}

#[component]
pub fn Bookings() -> impl IntoView {
    let (bookings, set_bookings) = create_signal(Vec::<Booking>::new());
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (loading, set_loading) = create_signal(true);

    let (selected_room, set_selected_room) = create_signal("".to_string());
    let (selected_cust, set_selected_cust) = create_signal("".to_string());
    let (check_in, set_check_in) = create_signal("".to_string());
    let (check_out, set_check_out) = create_signal("".to_string());

    let load_data = move || { 
        spawn_local(async move { 
            set_loading.set(true); 
            set_bookings.set(fetch_bookings().await); 
            set_rooms.set(fetch_rooms().await); 
            set_customers.set(fetch_customers("".to_string()).await); 
            set_loading.set(false); 
        }); 
    };
    create_effect(move |_| { load_data(); });

    let on_check_in = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let r_id = selected_room.get();
        let c_id = selected_cust.get();
        let room_opt = rooms.get_untracked().into_iter().find(|r| r.id.as_deref() == Some(&r_id));
        let cust_opt = customers.get_untracked().into_iter().find(|c| c.id.as_deref() == Some(&c_id));

        if let (Some(r), Some(c)) = (room_opt, cust_opt) {
            let new_booking = NewBooking {
                room_id: r_id, customer_id: c_id, customer_name: c.full_name,
                room_number: r.number, check_in_date: check_in.get(),
                check_out_date: check_out.get(), status: "Checked-In".to_string(),
            };
            spawn_local(async move {
                wait_for_bridge().await;
                let js_val = serde_wasm_bindgen::to_value(&new_booking).unwrap();
                match add_booking_js(js_val).await { 
                    Ok(_) => { load_data(); } 
                    Err(e) => logging::error!("Booking Error: {:?}", e), 
                }
            });
        }
    };

    view! {
        <div class="card">
            <h1>"Check-in & Bookings"</h1>
            <form on:submit=on_check_in class="card" style="background: #f9f9f9;">
                <div class="grid-form">
                    <div style="display: flex; flex-direction: column;">
                        <label>"Select Room"</label>
                        <select on:change=move |ev| set_selected_room.set(event_target_value(&ev)) prop:value=selected_room required>
                            <option value="">"Choose Room..."</option>
                            {move || {
                                rooms.get().into_iter()
                                    .filter(|r| r.status == "Available")
                                    .map(|r| {
                                        let r_id = r.id.clone().unwrap_or_default();
                                        let r_num = r.number.clone();
                                        let r_type = r.room_type.clone();
                                        view! { <option value=r_id>{r_num} " (" {r_type} ")" </option> }
                                    })
                                    .collect_view()
                            }}
                        </select>
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Select Customer"</label>
                        <select on:change=move |ev| set_selected_cust.set(event_target_value(&ev)) prop:value=selected_cust required>
                            <option value="">"Choose Customer..."</option>
                            {move || {
                                customers.get().into_iter()
                                    .map(|c| {
                                        let c_id = c.id.clone().unwrap_or_default();
                                        let c_name = c.full_name.clone();
                                        view! { <option value=c_id>{c_name}</option> }
                                    })
                                    .collect_view()
                            }}
                        </select>
                    </div>
                    <div style="display: flex; flex-direction: column;"><label>"Check-in"</label><input type="date" on:input=move |ev| set_check_in.set(event_target_value(&ev)) prop:value=check_in required /></div>
                    <div style="display: flex; flex-direction: column;"><label>"Check-out"</label><input type="date" on:input=move |ev| set_check_out.set(event_target_value(&ev)) prop:value=check_out required /></div>
                </div>
                <button type="submit" style="width: 100%; margin-top: 20px; background-color: #27ae60;">"Confirm Check-in"</button>
            </form>

            <h3>"Recent Stays"</h3>
            {move || if loading.get() { view! { <p>"Loading..."</p> }.into_view() } else { view! {
                <table>
                    <thead><tr style="background-color: #f2f2f2; text-align: left;"><th>"Guest"</th><th>"Room"</th><th>"Check-in"</th><th>"Status"</th></tr></thead>
                    <tbody>
                        <For each=move || bookings.get() key=|b| b.id.clone().unwrap_or_default() children=move |b| {
                            let b_name = b.customer_name.clone();
                            let b_room = b.room_number.clone();
                            let b_date = b.check_in_date.clone();
                            let b_status = b.status.clone();
                            view! { <tr><td>{b_name}</td><td>{b_room}</td><td>{b_date}</td><td><span style="color: #27ae60; font-weight: bold;">{b_status}</span></td></tr> }
                        } />
                    </tbody>
                </table>
            }.into_view() }}
        </div>
    }
}
