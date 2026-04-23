use leptos::*;
use crate::models::Booking;
use crate::api::{get_bookings_js, delete_booking_js};
use crate::utils::wait_for_bridge;

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

    let load_bookings = move || { spawn_local(async move { set_loading.set(true); set_bookings.set(fetch_bookings().await); set_loading.set(false); }); };
    create_effect(move |_| { load_bookings(); });

    let on_delete_final = move |id: String, room_id: String| {
        spawn_local(async move {
            wait_for_bridge().await;
            let _ = delete_booking_js(id, room_id).await;
            load_bookings();
        });
    };

    view! {
        <div class="card">
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
                                
                                view! {
                                    <tr>
                                        <td>{b.customer_name.clone()}</td>
                                        <td>"Room " {b.room_number.clone()}</td>
                                        <td><small>{b.check_in_date.clone()} " to " {b.check_out_date.clone()}</small></td>
                                        <td><span style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.8rem; background-color: {}; color: white;", if b.status == "Checked-In" { "#27ae60" } else { "#7f8c8d" })>{b.status.clone()}</span></td>
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
