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
    let (editing_id, set_editing_id) = create_signal(None::<String>);

    let load_rooms = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_rooms.set(fetch_rooms().await);
            set_loading.set(false);
        });
    };
    create_effect(move |_| { load_rooms(); });

    let on_add_room = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let new_room = NewRoom { number: number.get(), room_type: room_type.get(), status: "Available".to_string() };
        spawn_local(async move {
            wait_for_bridge().await;
            match serde_wasm_bindgen::to_value(&new_room) {
                Ok(js_val) => {
                    if let Some(id) = editing_id.get() { let _ = update_room_js(id, js_val).await; } 
                    else { let _ = add_room_js(js_val).await; }
                    set_editing_id.set(None); set_number.set("".to_string()); load_rooms();
                },
                Err(e) => logging::error!("Serialization Error: {:?}", e),
            }
        });
    };

    let on_edit = move |r: Room| {
        set_editing_id.set(r.id);
        set_number.set(r.number);
        set_room_type.set(r.room_type);
        window().scroll_to_with_x_and_y(0.0, 0.0);
    };

    let on_delete = move |id: String| {
        if window().confirm_with_message("Delete?").unwrap_or(false) {
            spawn_local(async move { wait_for_bridge().await; let _ = delete_room_js(id).await; load_rooms(); });
        }
    };

    view! {
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h1>"Rooms"</h1>
                {move || if editing_id.get().is_some() {
                    view! { <button on:click=move |_| set_editing_id.set(None) style="background:#6c757d;">"Cancel Edit"</button> }.into_view()
                } else { view! {}.into_view() }}
            </div>
            <form on:submit=on_add_room class="grid-form" style="margin-bottom: 20px;">
                <div style="display: flex; flex-direction: column;">
                    <label>"Number"</label>
                    <input type="text" on:input=move |ev| set_number.set(event_target_value(&ev)) prop:value=number required />
                </div>
                <div style="display: flex; flex-direction: column;">
                    <label>"Type"</label>
                    <select on:change=move |ev| set_room_type.set(event_target_value(&ev)) prop:value=room_type>
                        <option value="Delux">"Delux"</option>
                        <option value="AC">"AC"</option>
                        <option value="non-AC">"non-AC"</option>
                    </select>
                </div>
                <button type="submit" style="grid-column: 1 / -1;">
                    {move || if editing_id.get().is_some() { "Update Room" } else { "Add Room" }}
                </button>
            </form>
            {move || if loading.get() {
                view! { <p>"Loading rooms..."</p> }.into_view()
            } else {
                view! {
                    <table>
                        <thead><tr style="background-color: #f2f2f2; text-align: left;"><th>"Number"</th><th>"Type"</th><th>"Status"</th><th>"Actions"</th></tr></thead>
                        <tbody>
                            <For each=move || rooms.get() key=|room| room.id.clone().unwrap_or_default() children=move |room| {
                                let r_cloned = room.clone();
                                let id_cloned = room.id.clone().unwrap_or_default();
                                view! {
                                    <tr>
                                        <td>{room.number.clone()}</td>
                                        <td>{room.room_type.clone()}</td>
                                        <td><span style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.8rem; background-color: {}; color: white;", if room.status == "Available" { "#27ae60" } else { "#e67e22" })>{room.status.clone()}</span></td>
                                        <td>
                                            <button on:click=move |_| on_edit(r_cloned.clone()) style="padding: 5px 10px; margin-right: 5px; font-size: 0.8rem; background: #3498db;">"Edit"</button>
                                            <button on:click=move |_| on_delete(id_cloned.clone()) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Del"</button>
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
