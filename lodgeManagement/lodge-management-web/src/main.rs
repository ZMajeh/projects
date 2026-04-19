use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Customer {
    pub id: Option<String>,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub aadhaar: String,
    pub age: String,
    pub gender: String,
    pub photo_data: Option<String>,
    pub id_card_data: Option<String>,
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
    #[wasm_bindgen(catch, js_name = getCustomers)]
    async fn get_customers_js() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = addCustomer)]
    async fn add_customer_js(customer: JsValue) -> Result<JsValue, JsValue>;
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

fn validate_aadhaar_checksum(aadhaar: &str) -> bool {
    let clean_aadhaar = aadhaar.trim().replace(" ", "");
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
        if let Ok(year) = parts[2].parse::<i32>() {
            return Some((2026 - year).to_string());
        }
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
    match get_rooms_js().await {
        Ok(js_val) => serde_wasm_bindgen::from_value(js_val).unwrap_or_default(),
        Err(_) => vec![],
    }
}

async fn fetch_customers() -> Vec<Customer> {
    match get_customers_js().await {
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

    view! {
        <div style="display: flex; justify-content: center; align-items: center; height: 100vh; flex-direction: column; padding: 1rem;">
            <div class="container" style="max-width: 400px; text-align: center; width: 100%;">
                <h2>"Lodge Management Login"</h2>
                <form on:submit=on_submit>
                    <div style="margin-bottom: 15px; text-align: left;">
                        <label style="display: block; margin-bottom: 5px;">"Email"</label>
                        <input type="email" style="width: 100%;" on:input=move |ev| set_email.set(event_target_value(&ev)) prop:value=email required />
                    </div>
                    <div style="margin-bottom: 15px; text-align: left;">
                        <label style="display: block; margin-bottom: 5px;">"Password"</label>
                        <input type="password" style="width: 100%;" on:input=move |ev| set_password.set(event_target_value(&ev)) prop:value=password required />
                    </div>
                    {move || error.get().map(|err| view! { <p style="color: red; font-size: 0.9rem;">{err}</p> })}
                    <button type="submit" style="width: 100%;" disabled=loading>
                        {move || if loading.get() { "Logging in..." } else { "Login" }}
                    </button>
                </form>
            </div>
        </div>
    }
}

#[component]
fn DashboardHome() -> impl IntoView {
    view! {
        <div class="card">
            <h1>"Dashboard Overview"</h1>
            <p>"Welcome to your Lodge Management System. Use the sidebar to navigate."</p>
        </div>
    }
}

#[component]
fn Rooms() -> impl IntoView {
    let (rooms, set_rooms) = create_signal(Vec::<Room>::new());
    let (loading, set_loading) = create_signal(true);
    let (number, set_number) = create_signal("".to_string());
    let (room_type, set_room_type) = create_signal("Single".to_string());

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
        let new_room = Room { id: None, number: number.get(), room_type: room_type.get(), status: "Available".to_string() };
        spawn_local(async move {
            let js_val = serde_wasm_bindgen::to_value(&new_room).unwrap();
            match add_room_js(js_val).await {
                Ok(_) => { set_number.set("".to_string()); load_rooms(); }
                Err(e) => logging::error!("Error adding room: {:?}", e),
            }
        });
    };

    view! {
        <div class="card">
            <h1>"Rooms Management"</h1>
            <form on:submit=on_add_room class="grid-form" style="margin-bottom: 20px;">
                <div style="display: flex; flex-direction: column;">
                    <label>"Room Number"</label>
                    <input type="text" on:input=move |ev| set_number.set(event_target_value(&ev)) prop:value=number required />
                </div>
                <div style="display: flex; flex-direction: column;">
                    <label>"Type"</label>
                    <select on:change=move |ev| set_room_type.set(event_target_value(&ev)) prop:value=room_type>
                        <option value="Single">"Single"</option>
                        <option value="Double">"Double"</option>
                        <option value="Suite">"Suite"</option>
                    </select>
                </div>
                <button type="submit" style="grid-column: 1 / -1;">"Add Room"</button>
            </form>
            {move || if loading.get() { view! { <p>"Loading rooms..."</p> }.into_view() } else {
                view! {
                    <table>
                        <thead>
                            <tr style="background-color: #f2f2f2; text-align: left;">
                                <th style="padding: 12px; border: 1px solid #ddd;">"Number"</th>
                                <th style="padding: 12px; border: 1px solid #ddd;">"Type"</th>
                                <th style="padding: 12px; border: 1px solid #ddd;">"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For each=move || rooms.get() key=|room| room.id.clone().unwrap_or_default() children=|room| view! {
                                <tr>
                                    <td style="padding: 12px; border: 1px solid #ddd;">{room.number}</td>
                                    <td style="padding: 12px; border: 1px solid #ddd;">{room.room_type}</td>
                                    <td style="padding: 12px; border: 1px solid #ddd;">
                                        <span style=format!("padding: 4px 8px; border-radius: 4px; font-size: 0.8rem; background-color: {}; color: white;", if room.status == "Available" { "#27ae60" } else { "#e67e22" })>{room.status}</span>
                                    </td>
                                </tr>
                            } />
                        </tbody>
                    </table>
                }.into_view()
            }}
        </div>
    }
}

#[component]
fn Customers() -> impl IntoView {
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (loading, set_loading) = create_signal(true);
    let (name, set_name) = create_signal("".to_string());
    let (phone, set_phone) = create_signal("".to_string());
    let (email, set_email) = create_signal("".to_string());
    let (aadhaar, set_aadhaar) = create_signal("".to_string());
    let (age, set_age) = create_signal("".to_string());
    let (gender, set_gender) = create_signal("Male".to_string());
    
    let (photo, set_photo) = create_signal(None::<String>);
    let (id_card, set_id_card) = create_signal(None::<String>);
    let (camera_active, set_camera_active) = create_signal(false);
    let (capture_target, set_capture_target) = create_signal("photo");
    let (ocr_loading, set_ocr_loading) = create_signal(false);
    let (is_verified, set_is_verified) = create_signal(false);
    let (show_manual_verify, set_show_manual_verify) = create_signal(false);
    let (ocr_raw_text, set_ocr_raw_text) = create_signal("".to_string());

    let load_customers = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_customers.set(fetch_customers().await);
            set_loading.set(false);
        });
    };
    create_effect(move |_| { load_customers(); });

    let start_capture = move |target: &'static str| {
        set_capture_target.set(target);
        set_camera_active.set(true);
        spawn_local(async move { let _ = start_camera("cam-preview".to_string()).await; });
    };

    let process_id_ocr = move |data: String| {
        set_ocr_loading.set(true);
        spawn_local(async move {
            if let Ok(result_js) = extract_aadhaar_js(data).await {
                if let Some(obj) = js_sys::Object::try_from(&result_js) {
                    if let Ok(id_js) = js_sys::Reflect::get(obj, &JsValue::from_str("aadhaar")) {
                        if let Some(id) = id_js.as_string() { set_aadhaar.set(id); }
                    }
                    if let Ok(name_js) = js_sys::Reflect::get(obj, &JsValue::from_str("name")) {
                        if let Some(n) = name_js.as_string() { if name.get_untracked().is_empty() { set_name.set(n); } }
                    }
                    if let Ok(raw_js) = js_sys::Reflect::get(obj, &JsValue::from_str("raw_text")) {
                        if let Some(txt) = raw_js.as_string() { set_ocr_raw_text.set(txt); }
                    }
                    if let Ok(dob_js) = js_sys::Reflect::get(obj, &JsValue::from_str("dob")) {
                        if let Some(d) = dob_js.as_string() { if let Some(a) = calculate_age(&d) { set_age.set(a); } }
                    }
                    if let Ok(gender_js) = js_sys::Reflect::get(obj, &JsValue::from_str("gender")) {
                        if let Some(g) = gender_js.as_string() { set_gender.set(g); }
                    }
                }
            }
            set_ocr_loading.set(false);
        });
    };

    let do_capture = move |_: leptos::ev::MouseEvent| {
        spawn_local(async move {
            if let Ok(data_js) = take_snapshot("cam-preview".to_string()).await {
                if let Some(data) = data_js.as_string() {
                    if capture_target.get() == "photo" { set_photo.set(Some(data)); } 
                    else { set_id_card.set(Some(data.clone())); process_id_ocr(data); }
                }
            }
            let _ = stop_camera().await;
            set_camera_active.set(false);
        });
    };

    let on_file_upload = move |ev: leptos::ev::Event, target: &'static str| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                spawn_local(async move {
                    if let Ok(data_js) = read_file_as_data_url(file).await {
                        if let Some(data) = data_js.as_string() {
                            if target == "photo" { set_photo.set(Some(data)); } 
                            else { set_id_card.set(Some(data.clone())); process_id_ocr(data); }
                        }
                    }
                });
            }
        }
    };

    let on_verify_aadhaar = move |_| {
        let num = aadhaar.get();
        if !validate_aadhaar_checksum(&num) { window().alert_with_message("Checksum verification failed!").ok(); return; }
        spawn_local(async move {
            let _ = manual_verify_aadhaar(num.clone()).await;
            set_show_manual_verify.set(true);
        });
    };

    let on_add_customer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if !is_verified.get() { window().alert_with_message("Please verify Aadhaar before saving!").ok(); return; }
        let new_cust = Customer { id: None, full_name: name.get(), phone: phone.get(), email: email.get(), aadhaar: aadhaar.get(), age: age.get(), gender: gender.get(), photo_data: photo.get(), id_card_data: id_card.get() };
        spawn_local(async move {
            let js_val = serde_wasm_bindgen::to_value(&new_cust).unwrap();
            match add_customer_js(js_val).await {
                Ok(_) => { 
                    set_name.set("".to_string()); set_phone.set("".to_string()); set_email.set("".to_string()); 
                    set_aadhaar.set("".to_string()); set_age.set("".to_string()); set_gender.set("Male".to_string());
                    set_photo.set(None); set_id_card.set(None); set_is_verified.set(false); set_ocr_raw_text.set("".to_string());
                    load_customers(); 
                }
                Err(e) => logging::error!("Error adding customer: {:?}", e),
            }
        });
    };

    view! {
        <div class="card">
            <h1>"Customer Entry"</h1>
            <form on:submit=on_add_customer class="card" style="background: #f9f9f9;">
                <div class="grid-form">
                    <div style="display: flex; flex-direction: column;">
                        <label>"Full Name"</label>
                        <input type="text" on:input=move |ev| set_name.set(event_target_value(&ev)) prop:value=name required />
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Phone Number"</label>
                        <input type="tel" on:input=move |ev| set_phone.set(event_target_value(&ev)) prop:value=phone required />
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Age"</label>
                        <input type="number" on:input=move |ev| set_age.set(event_target_value(&ev)) prop:value=age required />
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Gender"</label>
                        <select on:change=move |ev| set_gender.set(event_target_value(&ev)) prop:value=gender>
                            <option value="Male">"Male"</option>
                            <option value="Female">"Female"</option>
                            <option value="Other">"Other"</option>
                        </select>
                    </div>
                    <div style="display: flex; flex-direction: column; grid-column: 1 / -1;">
                        <label>"Aadhaar Number"</label>
                        <div style="display: flex; gap: 5px;">
                            <input type="text" maxlength="12" on:input=move |ev| { set_aadhaar.set(event_target_value(&ev)); set_is_verified.set(false); } 
                                prop:value=aadhaar style=move || format!("border-color: {};", if is_verified.get() { "green" } else { "#ddd" }) required />
                            <button type="button" on:click=on_verify_aadhaar disabled=ocr_loading>"Verify"</button>
                        </div>
                        <span style="font-size: 0.7rem; color: #666;">{move || if is_verified.get() { "✅ Validated" } else { "Manual check required" }}</span>
                    </div>
                </div>
                <div class="grid-form" style="margin-top: 20px;">
                    <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;">
                        <p>"Photo"</p>
                        {move || photo.get().map(|d| view! { <img src=d style="width: 100%; max-height: 100px;" /> })}
                        <div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;">
                            <button type="button" on:click=move |_| start_capture("photo") style="font-size: 0.8rem;">"Camera"</button>
                            <input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "photo") style="font-size: 0.7rem;" />
                        </div>
                    </div>
                    <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;">
                        <p>"Aadhaar Scan"</p>
                        {move || id_card.get().map(|d| view! { <img src=d style="width: 100%; max-height: 100px;" /> })}
                        <div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;">
                            <button type="button" on:click=move |_| start_capture("id") style="font-size: 0.8rem;">"Camera"</button>
                            <input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "id") style="font-size: 0.7rem;" />
                        </div>
                    </div>
                </div>
                <button type="submit" style="width: 100%; margin-top: 20px; background-color: #27ae60;">"Save Verified Customer"</button>
            </form>

            {move || if show_manual_verify.get() {
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 3000;">
                        <div class="card" style="max-width: 400px; text-align: center; padding: 2rem;">
                            <h3>"Confirm Aadhaar Validity"</h3>
                            <p>"Official UIDAI site opened. ID copied to clipboard."</p>
                            <div style="display: flex; gap: 10px; margin-top: 20px;">
                                <button on:click=move |_| { set_is_verified.set(true); set_show_manual_verify.set(false); } style="background: green; flex: 1;">"YES - VALID"</button>
                                <button on:click=move |_| { set_is_verified.set(false); set_show_manual_verify.set(false); } style="background: red; flex: 1;">"NO - INVALID"</button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else { view! {}.into_view() }}

            {move || if camera_active.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.9); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 2000; padding: 1rem;"><video id="cam-preview" style="width: 100%; max-width: 500px; border: 2px solid white;"></video><div style="margin-top: 20px; display: flex; gap: 10px;"><button on:click=move |_| {
                spawn_local(async move {
                    if let Ok(data_js) = take_snapshot("cam-preview".to_string()).await {
                        if let Some(data) = data_js.as_string() {
                            if capture_target.get() == "photo" { set_photo.set(Some(data)); } 
                            else { set_id_card.set(Some(data.clone())); process_id_ocr(data); }
                        }
                    }
                    let _ = stop_camera().await;
                    set_camera_active.set(false);
                });
            } style="background: green;">"CAPTURE"</button><button on:click=move |_| {
                spawn_local(async move {
                    let _ = stop_camera().await;
                    set_camera_active.set(false);
                });
            } style="background: red;">"CLOSE"</button></div></div> }.into_view() } else { view! {}.into_view() }}
            <h3>"Directory"</h3>
            {move || if loading.get() { view! { <p>"Loading..."</p> }.into_view() } else { view! { <table><thead><tr style="background-color: #f2f2f2; text-align: left;"><th style="padding: 12px; border: 1px solid #ddd;">"Name"</th><th style="padding: 12px; border: 1px solid #ddd;">"Aadhaar"</th><th style="padding: 12px; border: 1px solid #ddd;">"Age/Gender"</th></tr></thead><tbody><For each=move || customers.get() key=|c| c.id.clone().unwrap_or_default() children=|c| view! { <tr><td style="padding: 12px; border: 1px solid #ddd;">{c.full_name}</td><td style="padding: 12px; border: 1px solid #ddd;">{c.aadhaar}</td><td style="padding: 12px; border: 1px solid #ddd;">{c.age} " / " {c.gender}</td></tr> } /></tbody></table> }.into_view() }}
        </div>
    }
}

#[component]
fn DashboardLayout(user: User, on_logout: Callback<()>) -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);
    let handle_logout = move |_| { clear_user(); spawn_local(async move { let _ = sign_out_user().await; on_logout.call(()); }); };
    view! { <div class="app-layout"><div class=move || format!("sidebar-overlay {}", if menu_open.get() { "show" } else { "" }) on:click=move |_| set_menu_open.set(false)></div><nav class=move || format!("sidebar {}", if menu_open.get() { "open" } else { "" })><h2>"Lodge Manager"</h2><A href="" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active" exact=true>"Overview"</A><A href="rooms" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Rooms"</A><A href="customers" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Customers"</A><div style="margin-top: auto; padding-top: 1rem; border-top: 1px solid #444; font-size: 0.8rem;"><p style="color: #bdc3c7; overflow: hidden; text-overflow: ellipsis;">{user.email}</p><button on:click=handle_logout style="background-color: #e74c3c; width: 100%; margin-top: 10px;">"Logout"</button></div></nav><main class="content"><header class="mobile-header"><button on:click=move |_| set_menu_open.update(|v| *v = !*v) style="background: none; color: black; font-size: 1.5rem; padding: 0;">"☰"</button><strong>"Lodge Manager"</strong><div style="width: 30px;"></div></header><Routes><Route path="" view=DashboardHome /><Route path="rooms" view=Rooms /><Route path="customers" view=Customers /></Routes></main></div> }
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
