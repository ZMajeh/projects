use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub email: String,
    pub uid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Room {
    pub id: Option<String>,
    pub number: String,
    pub room_type: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct NewRoom {
    pub number: String,
    pub room_type: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Customer {
    pub id: Option<String>,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub aadhaar: String,
    pub age: Option<String>,
    pub gender: Option<String>,
    pub photo_data: Option<String>,
    pub id_card_data: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Serialize)]
pub struct NewCustomer {
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub aadhaar: String,
    pub age: Option<String>,
    pub gender: Option<String>,
    pub photo_data: Option<String>,
    pub id_card_data: Option<String>,
    pub verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Booking {
    pub id: Option<String>,
    pub room_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub room_number: String,
    pub check_in_date: String,
    pub check_out_date: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct NewBooking {
    #[serde(rename = "roomId")]
    pub room_id: String,
    #[serde(rename = "customerId")]
    pub customer_id: String,
    #[serde(rename = "customerName")]
    pub customer_name: String,
    #[serde(rename = "roomNumber")]
    pub room_number: String,
    #[serde(rename = "checkInDate")]
    pub check_in_date: String,
    #[serde(rename = "checkOutDate")]
    pub check_out_date: String,
    pub status: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = loginUser)]
    async fn login_user(email: String, pass: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = signOutUser)]
    async fn sign_out_user() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = getRooms)]
    async fn get_rooms_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = addRoom)]
    async fn add_room_js(room: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = updateRoom)]
    async fn update_room_js(id: String, room: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = deleteRoom)]
    async fn delete_room_js(id: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = getCustomers)]
    async fn get_customers_js(search: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = addCustomer)]
    async fn add_customer_js(customer: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = updateCustomer)]
    async fn update_customer_js(id: String, customer: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = deleteCustomer)]
    async fn delete_customer_js(id: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = getBookings)]
    async fn get_bookings_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = addBooking)]
    async fn add_booking_js(booking: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = startCamera)]
    async fn start_camera(id: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = takeSnapshot)]
    async fn take_snapshot(id: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = stopCamera)]
    async fn stop_camera() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = extractAadhaar)]
    async fn extract_aadhaar_js(base64: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = readFileAsDataURL)]
    async fn read_file_as_data_url(file: web_sys::File) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = manualVerifyAadhaar)]
    async fn manual_verify_aadhaar(num: String) -> Result<JsValue, JsValue>;
}

fn is_bridge_ready() -> bool {
    if let Ok(ready) = js_sys::Reflect::get(&window(), &JsValue::from_str("bridgeReady")) {
        ready.as_bool().unwrap_or(false)
    } else { false }
}

async fn wait_for_bridge() {
    while !is_bridge_ready() { gloo_timers::future::TimeoutFuture::new(100).await; }
}

fn validate_aadhaar_checksum(aadhaar: &str) -> bool {
    let clean_aadhaar = aadhaar.trim().replace(" ", "").replace("-", "");
    if clean_aadhaar.len() != 12 || !clean_aadhaar.chars().all(|c| c.is_ascii_digit()) { return false; }
    let multiplication_table = [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],[1, 2, 3, 4, 0, 6, 7, 8, 9, 5],[2, 3, 4, 0, 1, 7, 8, 9, 5, 6],[3, 4, 0, 1, 2, 8, 9, 5, 6, 7],[4, 0, 1, 2, 3, 9, 5, 6, 7, 8],[5, 9, 8, 7, 6, 4, 3, 2, 1, 0],[6, 5, 9, 8, 7, 0, 4, 3, 2, 1],[7, 6, 5, 9, 8, 1, 0, 4, 3, 2],[8, 7, 6, 5, 9, 2, 1, 0, 4, 3],[9, 8, 7, 6, 5, 3, 2, 1, 0, 4]];
    let permutation_table = [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],[1, 5, 7, 6, 2, 8, 3, 0, 9, 4],[5, 8, 0, 3, 7, 9, 6, 1, 4, 2],[8, 9, 1, 6, 0, 4, 3, 5, 2, 7],[9, 4, 5, 3, 1, 2, 6, 8, 7, 0],[4, 2, 8, 6, 5, 7, 3, 9, 0, 1],[2, 7, 9, 3, 8, 0, 6, 4, 1, 5],[7, 0, 4, 6, 9, 1, 3, 2, 5, 8]];
    let mut c = 0;
    let digits: Vec<usize> = clean_aadhaar.chars().map(|d| d.to_digit(10).unwrap() as usize).collect();
    for (i, &digit) in digits.iter().rev().enumerate() { c = multiplication_table[c][permutation_table[i % 8][digit]]; }
    c == 0
}

fn calculate_age(dob: &str) -> Option<String> {
    let parts: Vec<&str> = dob.split('/').collect();
    if parts.len() == 3 {
        if let Ok(year) = parts[2].parse::<i32>() { return Some((2026 - year).to_string()); }
    }
    None
}

fn get_saved_user() -> Option<User> {
    let storage = window().local_storage().ok()??;
    let user_json = storage.get_item("user").ok()??;
    serde_json::from_str(&user_json).ok()
}

fn save_user(user: &User) {
    if let Ok(Some(s)) = window().local_storage() {
        let _ = s.set_item("user", &serde_json::to_string(user).unwrap_or_default());
    }
}

fn clear_user() {
    if let Ok(Some(s)) = window().local_storage() { let _ = s.remove_item("user"); }
}

async fn fetch_rooms() -> Vec<Room> {
    wait_for_bridge().await;
    match get_rooms_js().await {
        Ok(js_val) => serde_wasm_bindgen::from_value(js_val).unwrap_or_default(),
        Err(_) => vec![],
    }
}

async fn fetch_customers(search: String) -> Vec<Customer> {
    wait_for_bridge().await;
    match get_customers_js(search).await {
        Ok(js_val) => {
            match serde_wasm_bindgen::from_value::<Vec<Customer>>(js_val) {
                Ok(customers) => customers,
                Err(e) => { logging::error!("RUST ERROR: Failed to deserialize customers: {:?}", e); vec![] }
            }
        },
        Err(_) => vec![],
    }
}

async fn fetch_bookings() -> Vec<Booking> {
    wait_for_bridge().await;
    match get_bookings_js().await {
        Ok(js_val) => serde_wasm_bindgen::from_value(js_val).unwrap_or_default(),
        Err(_) => vec![],
    }
}

#[component]
fn Login(on_login: Callback<User>) -> impl IntoView {
    let (email, set_email) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);
        let email_val = email.get();
        let pass_val = password.get();
        spawn_local(async move {
            wait_for_bridge().await;
            match login_user(email_val, pass_val).await {
                Ok(user_js) => {
                    if let Ok(user) = serde_wasm_bindgen::from_value::<User>(user_js) {
                        save_user(&user);
                        on_login.call(user);
                    }
                }
                Err(_) => { set_error.set(Some("Login failed. Check credentials.".to_string())); }
            }
            set_loading.set(false);
        });
    };
    view! { <div style="display: flex; justify-content: center; align-items: center; height: 100vh; flex-direction: column; padding: 1rem;"><div class="container" style="max-width: 400px; text-align: center; width: 100%;"><h2>"Lodge Management Login"</h2><form on:submit=on_submit><div style="margin-bottom: 15px; text-align: left;"><label style="display: block; margin-bottom: 5px;">"Email"</label><input type="email" style="width: 100%;" on:input=move |ev| set_email.set(event_target_value(&ev)) prop:value=email required /></div><div style="margin-bottom: 15px; text-align: left;"><label style="display: block; margin-bottom: 5px;">"Password"</label><input type="password" style="width: 100%;" on:input=move |ev| set_password.set(event_target_value(&ev)) prop:value=password required /></div>{move || error.get().map(|err| view! { <p style="color: red; font-size: 0.9rem;">{err}</p> })}<button type="submit" style="width: 100%;" disabled=loading>{move || if loading.get() { "Logging in..." } else { "Login" }}</button></form></div></div> }
}

#[component]
fn DashboardHome() -> impl IntoView {
    view! { <div class="card"><h1>"Dashboard Overview"</h1><p>"Welcome to your Lodge Management System. Use the sidebar to navigate."</p></div> }
}

#[component]
fn Rooms() -> impl IntoView {
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (loading, set_loading) = create_signal(true);
    let (number, set_number) = create_signal("".to_string());
    let (room_type, set_room_type) = create_signal("Delux".to_string());
    let (editing_id, set_editing_id) = create_signal(None::<String>);
    let load_rooms = move || { spawn_local(async move { set_loading.set(true); set_rooms.set(fetch_rooms().await); set_loading.set(false); }); };
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
    let on_edit = move |r: Room| { set_editing_id.set(r.id); set_number.set(r.number); set_room_type.set(r.room_type); window().scroll_to_with_x_and_y(0.0, 0.0); };
    let on_delete = move |id: String| { if window().confirm_with_message("Delete?").unwrap_or(false) { spawn_local(async move { wait_for_bridge().await; let _ = delete_room_js(id).await; load_rooms(); }); } };
    view! { <div class="card"><div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h1>"Rooms"</h1>{move || if editing_id.get().is_some() { view! { <button on:click=move |_| set_editing_id.set(None) style="background:#6c757d;">"Cancel"</button> }.into_view() } else { view! {}.into_view() }}</div><form on:submit=on_add_room class="grid-form" style="margin-bottom: 20px;"><div style="display: flex; flex-direction: column;"><label>"Number"</label><input type="text" on:input=move |ev| set_number.set(event_target_value(&ev)) prop:value=number required /></div><div style="display: flex; flex-direction: column;"><label>"Type"</label><select on:change=move |ev| set_room_type.set(event_target_value(&ev)) prop:value=room_type><option value="Delux">"Delux"</option><option value="AC">"AC"</option><option value="non-AC">"non-AC"</option></select></div><button type="submit" style="grid-column: 1 / -1;">{move || if editing_id.get().is_some() { "Update Room" } else { "Add Room" }}</button></form>{move || if loading.get() { view! { <p>"Loading rooms..."</p> }.into_view() } else { view! { <table><thead><tr style="background-color: #f2f2f2; text-align: left;"><th>"Number"</th><th>"Type"</th><th>"Status"</th><th>"Actions"</th></tr></thead><tbody><For each=move || rooms.get() key=|room| room.id.clone().unwrap_or_default() children=move |room| { let r_cloned = room.clone(); let id_cloned = room.id.clone().unwrap_or_default(); view! { <tr><td>{room.number.clone()}</td><td>{room.room_type.clone()}</td><td><span style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.8rem; background-color: {}; color: white;", if room.status == "Available" { "#27ae60" } else { "#e67e22" })>{room.status.clone()}</span></td><td><button on:click=move |_| on_edit(r_cloned.clone()) style="padding: 5px 10px; margin-right: 5px; font-size: 0.8rem; background: #3498db;">"Edit"</button><button on:click=move |_| on_delete(id_cloned.clone()) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Del"</button></td></tr> } } /></tbody></table> }.into_view() }}</div> }
}

#[component]
fn Bookings() -> impl IntoView {
    let (bookings, set_bookings) = create_signal(Vec::<Booking>::new());
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (loading, set_loading) = create_signal(true);

    let (selected_room, set_selected_room) = create_signal("".to_string());
    let (selected_cust, set_selected_cust) = create_signal("".to_string());
    let (check_in, set_check_in) = create_signal("".to_string());
    let (check_out, set_check_out) = create_signal("".to_string());

    let load_data = move || { spawn_local(async move { set_loading.set(true); set_bookings.set(fetch_bookings().await); set_rooms.set(fetch_rooms().await); set_customers.set(fetch_customers("".to_string()).await); set_loading.set(false); }); };
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
                match add_booking_js(js_val).await { Ok(_) => { load_data(); } Err(e) => logging::error!("Booking Error: {:?}", e), }
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
                            <For each=move || rooms.get().into_iter().filter(|r| r.status == "Available").collect::<Vec<_>>() 
                                 key=|r| r.id.clone().unwrap_or_default()
                                 children=|r| view! { <option value=r.id.clone()>{r.number.clone()} " (" {r.room_type.clone()} ")" </option> } />
                        </select>
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Select Customer"</label>
                        <select on:change=move |ev| set_selected_cust.set(event_target_value(&ev)) prop:value=selected_cust required>
                            <option value="">"Choose Customer..."</option>
                            <For each=move || customers.get() 
                                 key=|c| c.id.clone().unwrap_or_default()
                                 children=|c| view! { <option value=c.id.clone()>{c.full_name.clone()}</option> } />
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
                    <tbody><For each=move || bookings.get() key=|b| b.id.clone().unwrap_or_default() children=|b| view! { <tr><td>{b.customer_name.clone()}</td><td>{b.room_number.clone()}</td><td>{b.check_in_date.clone()}</td><td><span style="color: #27ae60; font-weight: bold;">{b.status.clone()}</span></td></tr> } /></tbody>
                </table>
            }.into_view() }}
        </div>
    }
}

#[component]
fn Customers() -> impl IntoView {
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (loading, set_loading) = create_signal(true);
    let (search_query, set_search_query) = create_signal("".to_string());
    let (name, set_name) = create_signal("".to_string());
    let (phone, set_phone) = create_signal("".to_string());
    let (email, set_email) = create_signal("".to_string());
    let (aadhaar, set_aadhaar) = create_signal("".to_string());
    let (age, set_age) = create_signal("".to_string());
    let (gender, set_gender) = create_signal("Male".to_string());
    let (editing_id, set_editing_id) = create_signal(None::<String>);
    let (photo, set_photo) = create_signal(None::<String>);
    let (id_card, set_id_card) = create_signal(None::<String>);
    let (camera_active, set_camera_active) = create_signal(false);
    let (capture_target, set_capture_target) = create_signal("photo");
    let (ocr_loading, set_ocr_loading) = create_signal(false);
    let (is_verified, set_is_verified) = create_signal(false);
    let (show_manual_verify, set_show_manual_verify) = create_signal(false);
    let (manual_verify_target_id, set_manual_verify_target_id) = create_signal(None::<String>);
    let load_customers = move |q: String| { spawn_local(async move { set_loading.set(true); set_customers.set(fetch_customers(q).await); set_loading.set(false); }); };
    create_effect(move |_| { let q = search_query.get(); spawn_local(async move { gloo_timers::future::TimeoutFuture::new(500).await; if q == search_query.get_untracked() { load_customers(q); } }); });
    let start_capture = move |target: &'static str| { set_capture_target.set(target); set_camera_active.set(true); spawn_local(async move { wait_for_bridge().await; let _ = start_camera("cam-preview".to_string()).await; }); };
    let process_id_ocr = move |data: String| { set_ocr_loading.set(true); spawn_local(async move { wait_for_bridge().await; if let Ok(result_js) = extract_aadhaar_js(data).await { if let Some(obj) = js_sys::Object::try_from(&result_js) { if let Ok(id_js) = js_sys::Reflect::get(obj, &JsValue::from_str("aadhaar")) { if let Some(id) = id_js.as_string() { set_aadhaar.set(id); } } if let Ok(name_js) = js_sys::Reflect::get(obj, &JsValue::from_str("name")) { if let Some(n) = name_js.as_string() { if name.get_untracked().is_empty() { set_name.set(n); } } } if let Ok(dob_js) = js_sys::Reflect::get(obj, &JsValue::from_str("dob")) { if let Some(d) = dob_js.as_string() { if let Some(a) = calculate_age(&d) { set_age.set(a); } } } if let Ok(gender_js) = js_sys::Reflect::get(obj, &JsValue::from_str("gender")) { if let Some(g) = gender_js.as_string() { set_gender.set(g); } } } } set_ocr_loading.set(false); }); };
    let on_file_upload = move |ev: leptos::ev::Event, target: &'static str| { let input: web_sys::HtmlInputElement = event_target(&ev); if let Some(files) = input.files() { if let Some(file) = files.get(0) { spawn_local(async move { wait_for_bridge().await; if let Ok(data_js) = read_file_as_data_url(file).await { if let Some(data) = data_js.as_string() { if target == "photo" { set_photo.set(Some(data)); } else { set_id_card.set(Some(data.clone())); process_id_ocr(data); } } } }); } } };
    let on_verify_trigger = move |num: String, id: Option<String>| { set_manual_verify_target_id.set(id); spawn_local(async move { wait_for_bridge().await; let _ = manual_verify_aadhaar(num).await; set_show_manual_verify.set(true); }); };
    let on_manual_confirm = move |is_valid: bool| { let target_id = manual_verify_target_id.get(); set_show_manual_verify.set(false); if is_valid { if let Some(id) = target_id { spawn_local(async move { wait_for_bridge().await; let patch = serde_json::json!({ "verified": true }); let js_val = serde_wasm_bindgen::to_value(&patch).unwrap(); let _ = update_customer_js(id, js_val).await; load_customers(search_query.get_untracked()); }); } else { set_is_verified.set(true); } } };
    let on_add_customer = move |ev: leptos::ev::SubmitEvent| { ev.prevent_default(); if !is_verified.get() && editing_id.get().is_none() { window().alert_with_message("Please verify Aadhaar first!").ok(); return; } let cust_data = NewCustomer { full_name: name.get(), phone: phone.get(), email: email.get(), aadhaar: aadhaar.get(), age: Some(age.get()), gender: Some(gender.get()), photo_data: photo.get(), id_card_data: id_card.get(), verified: is_verified.get() }; spawn_local(async move { wait_for_bridge().await; let js_val = serde_wasm_bindgen::to_value(&cust_data).unwrap(); if let Some(id) = editing_id.get() { let _ = update_customer_js(id, js_val).await; } else { let _ = add_customer_js(js_val).await; } set_editing_id.set(None); set_name.set("".to_string()); set_phone.set("".to_string()); set_email.set("".to_string()); set_aadhaar.set("".to_string()); set_age.set("".to_string()); set_gender.set("Male".to_string()); set_photo.set(None); set_id_card.set(None); set_is_verified.set(false); load_customers(search_query.get_untracked()); }); };
    let on_edit = move |c: Customer| { set_editing_id.set(c.id.clone()); set_name.set(c.full_name); set_phone.set(c.phone); set_email.set(c.email); set_aadhaar.set(c.aadhaar); set_age.set(c.age.unwrap_or_default()); set_gender.set(c.gender.unwrap_or_else(|| "Male".to_string())); set_photo.set(c.photo_data); set_id_card.set(c.id_card_data); set_is_verified.set(c.verified); window().scroll_to_with_x_and_y(0.0, 0.0); };
    let on_delete = move |id: String| { if window().confirm_with_message("Delete?").unwrap_or(false) { spawn_local(async move { wait_for_bridge().await; let _ = delete_customer_js(id).await; load_customers(search_query.get_untracked()); }); } };
    view! { <div class="card"><div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"><h1>"Customers"</h1>{move || if editing_id.get().is_some() { view! { <button on:click=move |_| set_editing_id.set(None) style="background:#6c757d;">"Cancel"</button> }.into_view() } else { view! {}.into_view() }}</div><form on:submit=on_add_customer class="card" style="background: #f9f9f9;"><div class="grid-form"><div style="display: flex; flex-direction: column;"><label>"Full Name"</label><input type="text" on:input=move |ev| set_name.set(event_target_value(&ev)) prop:value=name required /></div><div style="display: flex; flex-direction: column;"><label>"Phone"</label><input type="tel" on:input=move |ev| set_phone.set(event_target_value(&ev)) prop:value=phone required /></div><div style="display: flex; flex-direction: column;"><label>"Age"</label><input type="number" on:input=move |ev| set_age.set(event_target_value(&ev)) prop:value=age required /></div><div style="display: flex; flex-direction: column;"><label>"Gender"</label><select on:change=move |ev| set_gender.set(event_target_value(&ev)) prop:value=gender><option value="Male">"Male"</option><option value="Female">"Female"</option><option value="Other">"Other"</option></select></div><div style="display: flex; flex-direction: column; grid-column: 1 / -1;"><label>"Aadhaar"</label><div style="display: flex; gap: 5px;"><input type="text" maxlength="12" on:input=move |ev| { set_aadhaar.set(event_target_value(&ev)); set_is_verified.set(false); } prop:value=aadhaar style=move || format!("border-color: {};", if is_verified.get() { "green" } else { "#ddd" }) required /><button type="button" on:click=move |_| on_verify_trigger(aadhaar.get(), None) disabled=ocr_loading>"Verify"</button></div></div></div><div class="grid-form" style="margin-top: 20px;"><div style="text-align: center; border: 1px dashed #ccc; padding: 10px;"><p>"Photo"</p>{move || photo.get().map(|d| view! { <img src=d style="width: 100%; max-height: 100px;" /> })}<div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;"><button type="button" on:click=move |_| start_capture("photo") style="font-size: 0.8rem;">"Camera"</button><input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "photo") style="font-size: 0.7rem;" /></div></div><div style="text-align: center; border: 1px dashed #ccc; padding: 10px;"><p>"ID Scan"</p>{move || id_card.get().map(|d| view! { <img src=d style="width: 100%; max-height: 100px;" /> })}<div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;"><button type="button" on:click=move |_| start_capture("id") style="font-size: 0.8rem;">"Camera"</button><input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "id") style="font-size: 0.7rem;" /></div></div></div><button type="submit" style="width: 100%; margin-top: 20px; background-color: #27ae60;">{move || if editing_id.get().is_some() { "Update Customer" } else { "Save Verified Customer" }}</button></form><div style="margin: 2rem 0;"><input type="text" placeholder="Search name/Aadhaar/phone..." on:input=move |ev| set_search_query.set(event_target_value(&ev)) style="padding: 1rem; font-size: 1.1rem; border: 2px solid var(--primary);" /></div><h3>"Directory"</h3>{move || if loading.get() { view! { <p>"Loading..."</p> }.into_view() } else { view! { <table><thead><tr style="background-color: #f2f2f2; text-align: left;"><th>"Name"</th><th>"Aadhaar"</th><th>"Contact"</th><th>"Status"</th><th>"Actions"</th></tr></thead><tbody><For each=move || customers.get() key=|c| c.id.clone().unwrap_or_default() children=move |c| { let c_cloned = c.clone(); let id_cloned = c.id.clone().unwrap_or_default(); let num_cloned = c.aadhaar.clone(); view! { <tr><td><strong>{c.full_name.clone()}</strong><br/><small>{c.age.clone().unwrap_or_else(|| "??".to_string())} {c.gender.clone().unwrap_or_else(|| "??".to_string())}</small></td><td>{c.aadhaar.clone()}</td><td>{c.phone.clone()}<br/><small style="color: #666;">{c.email.clone()}</small></td><td>{if c.verified { view! { <span style="color: green; font-weight: bold;">"✅ Verified"</span> }.into_view() } else { view! { <button on:click=move |_| on_verify_trigger(num_cloned.clone(), Some(id_cloned.clone())) style="padding: 4px 8px; font-size: 0.7rem; background: #f39c12;">"Verify Now"</button> }.into_view() }}</td><td><button on:click=move |_| on_edit(c_cloned.clone()) style="padding: 5px 10px; margin-right: 5px; font-size: 0.8rem; background: #3498db;">"Edit"</button><button on:click=move |_| on_delete(id_cloned.clone()) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Del"</button></td></tr> } } /></tbody></table> }.into_view() }}{move || if show_manual_verify.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 3000;"><div class="card" style="max-width: 400px; text-align: center; padding: 2rem;"><h3>"Confirm Aadhaar"</h3><p>"Official UIDAI site opened. ID copied."</p><div style="display: flex; gap: 10px; margin-top: 20px;"><button on:click=move |_| on_manual_confirm(true) style="background: green; flex: 1;">"YES"</button><button on:click=move |_| on_manual_confirm(false) style="background: red; flex: 1;">"NO"</button></div></div></div> }.into_view() } else { view! {}.into_view() }}{move || if camera_active.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.9); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 2000; padding: 1rem;"><video id="cam-preview" style="width: 100%; max-width: 500px; border: 2px solid white;"></video><div style="margin-top: 20px; display: flex; gap: 10px;"><button on:click=move |_: leptos::ev::MouseEvent| { spawn_local(async move { wait_for_bridge().await; if let Ok(data_js) = take_snapshot("cam-preview".to_string()).await { if let Some(data) = data_js.as_string() { if capture_target.get() == "photo" { set_photo.set(Some(data)); } else { set_id_card.set(Some(data.clone())); process_id_ocr(data); } } } let _ = stop_camera().await; set_camera_active.set(false); }); } style="background: green;">"CAPTURE"</button><button on:click=move |_: leptos::ev::MouseEvent| { spawn_local(async move { let _ = stop_camera().await; set_camera_active.set(false); }); } style="background: red;">"CLOSE"</button></div></div> }.into_view() } else { view! {}.into_view() }}</div> }
}

#[component]
fn DashboardLayout(user: User, on_logout: Callback<()>) -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);
    let handle_logout = move |_| { clear_user(); spawn_local(async move { wait_for_bridge().await; let _ = sign_out_user().await; on_logout.call(()); }); };
    view! { <div class="app-layout"><div class=move || format!("sidebar-overlay {}", if menu_open.get() { "show" } else { "" }) on:click=move |_| set_menu_open.set(false)></div><nav class=move || format!("sidebar {}", if menu_open.get() { "open" } else { "" })><h2>"Lodge Manager"</h2><A href="" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active" exact=true>"Overview"</A><A href="rooms" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Rooms"</A><A href="customers" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Customers"</A><A href="bookings" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Bookings"</A><div style="margin-top: auto; padding-top: 1rem; border-top: 1px solid #444; font-size: 0.8rem;"><p style="color: #bdc3c7; overflow: hidden; text-overflow: ellipsis;">{user.email}</p><button on:click=handle_logout style="background-color: #e74c3c; width: 100%; margin-top: 10px;">"Logout"</button></div></nav><main class="content"><header class="mobile-header"><button on:click=move |_| set_menu_open.update(|v| *v = !*v) style="background: none; color: black; font-size: 1.5rem; padding: 0;">"☰"</button><strong>"Lodge Manager"</strong><div style="width: 30px;"></div></header><Routes><Route path="" view=DashboardHome /><Route path="rooms" view=Rooms /><Route path="customers" view=Customers /><Route path="bookings" view=Bookings /></Routes></main></div> }
}

#[component]
fn App() -> impl IntoView {
    let (user, set_user) = create_signal(get_saved_user());
    view! { <Router><main>{move || match user.get() { Some(u) => view! { <DashboardLayout user=u on_logout=Callback::new(move |_| set_user.set(None))/> }.into_view(), None => view! { <Login on_login=Callback::new(move |u| set_user.set(Some(u)))/> }.into_view(), }}</main></Router> }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
