use leptos::*;
use crate::models::{Booking, NewBooking, Room, Customer};
use crate::api::{get_bookings_js, add_booking_js, update_booking_js, delete_booking_js};
use crate::utils::wait_for_bridge;
use crate::components::rooms::fetch_rooms;
use crate::components::customers::fetch_customers;

pub async fn fetch_bookings() -> Vec<Booking> {
    wait_for_bridge().await;
    match get_bookings_js().await {
        Ok(js_val) => {
            match serde_wasm_bindgen::from_value::<Vec<Booking>>(js_val) {
                Ok(bookings) => bookings,
                Err(e) => { logging::error!("RUST ERROR: Failed to deserialize bookings: {:?}", e); vec![] }
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

    let (editing_id, set_editing_id) = create_signal(None::<String>);
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
            let booking_data = NewBooking {
                room_id: r_id, customer_id: c_id, customer_name: c.full_name,
                room_number: r.number, check_in_date: check_in.get(),
                check_out_date: check_out.get(), status: "Checked-In".to_string(),
            };
            spawn_local(async move {
                wait_for_bridge().await;
                let js_val = serde_wasm_bindgen::to_value(&booking_data).unwrap();
                if let Some(id) = editing_id.get() {
                    let _ = update_booking_js(id, js_val).await;
                } else {
                    let _ = add_booking_js(js_val).await;
                }
                set_editing_id.set(None);
                set_selected_room.set("".to_string());
                set_selected_cust.set("".to_string());
                load_data();
            });
        }
    };

    let on_edit = move |b: Booking| {
        set_editing_id.set(b.id);
        set_selected_room.set(b.room_id);
        set_selected_cust.set(b.customer_id);
        set_check_in.set(b.check_in_date);
        set_check_out.set(b.check_out_date);
        window().scroll_to_with_x_and_y(0.0, 0.0);
    };

    let on_delete = move |b: Booking| {
        if window().confirm_with_message("Delete stay and mark room as Available?").unwrap_or(false) {
            let id = b.id.clone().unwrap_or_default();
            let room_id = b.room_id.clone();
            spawn_local(async move {
                wait_for_bridge().await;
                let _ = delete_booking_js(id, room_id).await;
                load_data();
            });
        }
    };

    view! {
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h1>"Check-in & Bookings"</h1>
                {move || if editing_id.get().is_some() {
                    view! { <button on:click=move |_| set_editing_id.set(None) style="background:#6c757d;">"Cancel Edit"</button> }.into_view()
                } else { view! {}.into_view() }}
            </div>
            
            <form on:submit=on_check_in class="card" style="background: #f9f9f9;">
                <div class="grid-form">
                    <div style="display: flex; flex-direction: column;">
                        <label>"Select Room"</label>
                        <select on:change=move |ev| set_selected_room.set(event_target_value(&ev)) prop:value=selected_room required>
                            <option value="">"Choose Room..."</option>
                            {move || {
                                rooms.get().into_iter()
                                    .filter(|r| r.status == "Available" || (editing_id.get().is_some() && Some(r.id.clone().unwrap_or_default()) == Some(selected_room.get())))
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
                <button type="submit" style="width: 100%; margin-top: 20px; background-color: #27ae60;">
                    {move || if editing_id.get().is_some() { "Update Stay" } else { "Confirm Check-in" }}
                </button>
            </form>

            <h3>"Recent Stays"</h3>
            {move || if loading.get() { view! { <p>"Loading..."</p> }.into_view() } else { view! {
                <table>
                    <thead><tr style="background-color: #f2f2f2; text-align: left;"><th>"Guest"</th><th>"Room"</th><th>"Check-in"</th><th>"Status"</th><th>"Actions"</th></tr></thead>
                    <tbody>
                        <For each=move || bookings.get() key=|b| b.id.clone().unwrap_or_default() children=move |b| {
                            let b_cloned = b.clone();
                            view! { 
                                <tr>
                                    <td>{b.customer_name.clone()}</td>
                                    <td>{b.room_number.clone()}</td>
                                    <td>{b.check_in_date.clone()}</td>
                                    <td><span style="color: #27ae60; font-weight: bold;">{b.status.clone()}</span></td>
                                    <td style="white-space: nowrap;">
                                        {
                                            let b_edit = b_cloned.clone();
                                            view! { <button on:click=move |_| on_edit(b_edit.clone()) style="padding: 5px 10px; margin-right: 5px; font-size: 0.8rem; background: #3498db;">"Edit"</button> }
                                        }
                                        {
                                            let b_del = b_cloned.clone();
                                            view! { <button on:click=move |_| on_delete(b_del.clone()) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Del"</button> }
                                        }
                                    </td>
                                </tr> 
                            }
                        } />
                    </tbody>
                </table>
            }.into_view() }}
        </div>
    }
}
