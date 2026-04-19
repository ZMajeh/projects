use leptos::*;
use crate::models::{Room, NewRoom};
use crate::api::{get_rooms_js, add_room_js, update_room_js, delete_room_js};
use crate::utils::wait_for_bridge;

pub async fn fetch_rooms() -> Vec<Room> {
    wait_for_bridge().await;
    match get_rooms_js().await {
        Ok(js_val) => serde_wasm_bindgen::from_value(js_val).unwrap_or_default(),
        Err(_) => vec![],
    }
}

#[component]
pub fn Rooms() -> impl IntoView {
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (loading, set_loading) = create_signal(true);
    let (number, set_number) = create_signal("".to_string());
    let (room_type, set_room_type) = create_signal("Delux".to_string());
    let (price, set_price) = create_signal("".to_string());
    let (editing_id, set_editing_id) = create_signal(None::<String>);
    let (confirm_delete_id, set_confirm_delete_id) = create_signal(None::<String>);

    let load_rooms = move || { spawn_local(async move { set_loading.set(true); set_rooms.set(fetch_rooms().await); set_loading.set(false); }); };
    create_effect(move |_| { load_rooms(); });

    let on_add_room = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        // Use get_untracked to avoid reactive warning in action handler
        let num_val = number.get_untracked();
        let type_val = room_type.get_untracked();
        let price_val = price.get_untracked().parse::<f64>().unwrap_or(0.0);
        let edit_id_val = editing_id.get_untracked();

        logging::log!("RUST: Submitting Room: {} ({}) - ₹{}", num_val, type_val, price_val);

        let new_room = NewRoom { 
            number: num_val, 
            room_type: type_val, 
            status: "Available".to_string(), 
            price: price_val 
        };
        
        spawn_local(async move {
            logging::log!("RUST: Waiting for bridge...");
            wait_for_bridge().await;
            logging::log!("RUST: Bridge ready, sending to JS...");
            match serde_wasm_bindgen::to_value(&new_room) {
                Ok(js_val) => {
                    if let Some(id) = edit_id_val { 
                        logging::log!("RUST: Updating Room ID: {}", id);
                        let _ = update_room_js(id, js_val).await; 
                    } 
                    else { 
                        logging::log!("RUST: Adding New Room");
                        let _ = add_room_js(js_val).await; 
                    }
                    set_editing_id.set(None); 
                    set_number.set("".to_string()); 
                    set_price.set("".to_string()); 
                    load_rooms();
                },
                Err(e) => logging::error!("RUST ERROR: Serialization Error: {:?}", e),
            }
        });
    };

    let on_edit = move |r: Room| {
        set_editing_id.set(r.id);
        set_number.set(r.number);
        set_room_type.set(r.room_type);
        set_price.set(r.price.to_string());
        window().scroll_to_with_x_and_y(0.0, 0.0);
    };

    let on_delete_final = move |id: String| { spawn_local(async move { wait_for_bridge().await; let _ = delete_room_js(id).await; load_rooms(); }); };

    view! {
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h1>"Rooms"</h1>
                {move || if editing_id.get().is_some() { view! { <button on:click=move |_| set_editing_id.set(None) style="background:#6c757d;">"Cancel Edit"</button> }.into_view() } else { view! {}.into_view() }}
            </div>
            <form on:submit=on_add_room class="grid-form" style="margin-bottom: 20px;">
                <div style="display: flex; flex-direction: column;"><label>"Number"</label><input type="text" on:input=move |ev| set_number.set(event_target_value(&ev)) prop:value=number required /></div>
                <div style="display: flex; flex-direction: column;"><label>"Type"</label><select on:change=move |ev| set_room_type.set(event_target_value(&ev)) prop:value=room_type><option value="Delux">"Delux"</option><option value="AC">"AC"</option><option value="non-AC">"non-AC"</option></select></div>
                <div style="display: flex; flex-direction: column;"><label>"Price (per day)"</label><input type="number" on:input=move |ev| set_price.set(event_target_value(&ev)) prop:value=price required /></div>
                <button type="submit" style="grid-column: 1 / -1;">{move || if editing_id.get().is_some() { "Update Room" } else { "Add Room" }}</button>
            </form>
            {move || if loading.get() { view! { <p>"Loading rooms..."</p> }.into_view() } else { view! {
                <table>
                    <thead><tr style="background-color: #f2f2f2; text-align: left;"><th>"Number"</th><th>"Type"</th><th>"Price"</th><th>"Status"</th><th>"Actions"</th></tr></thead>
                    <tbody><For each=move || rooms.get() key=|room| room.id.clone().unwrap_or_default() children=move |room| {
                        let r_cloned = room.clone();
                        let id_c = room.id.clone().unwrap_or_default();
                        view! {
                            <tr>
                                <td>{room.number.clone()}</td>
                                <td>{room.room_type.clone()}</td>
                                <td>"₹" {room.price}</td>
                                <td><span style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.8rem; background-color: {}; color: white;", if room.status == "Available" { "#27ae60" } else { "#e74c3c" })>{room.status.clone()}</span></td>
                                <td>{move || if confirm_delete_id.get() == Some(id_c.clone()) { let id_final = id_c.clone(); view! { <div style="display: flex; gap: 5px; align-items: center;"><span style="font-size: 0.7rem; color: red;">"Sure?"</span><button on:click=move |_| on_delete_final(id_final.clone()) style="padding: 2px 8px; font-size: 0.7rem; background: #e74c3c;">"YES"</button><button on:click=move |_| set_confirm_delete_id.set(None) style="padding: 2px 8px; font-size: 0.7rem; background: #6c757d;">"NO"</button></div> }.into_view() } else { let r_edit = r_cloned.clone(); let id_del = id_c.clone(); view! { <div style="display: flex; gap: 5px;"><button on:click=move |_| on_edit(r_edit.clone()) style="padding: 5px 10px; font-size: 0.8rem; background: #3498db;">"Edit"</button><button on:click=move |_| set_confirm_delete_id.set(Some(id_del.clone())) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Del"</button></div> }.into_view() }}</td>
                            </tr>
                        }
                    } /></tbody>
                </table>
            }.into_view() }}
        </div>
    }
}
