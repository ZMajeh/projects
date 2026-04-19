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
    pub photo_data: Option<String>, // Base64
    pub id_card_data: Option<String>, // Base64
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

    #[wasm_bindgen(js_name = takeSnapshot)]
    fn take_snapshot(id: String) -> String;

    #[wasm_bindgen(js_name = stopCamera)]
    fn stop_camera();
}

// Aadhaar Verhoeff Validation
fn validate_aadhaar(aadhaar: &str) -> bool {
    if aadhaar.len() != 12 || !aadhaar.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    
    let multiplication_table = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        [1, 2, 3, 4, 0, 6, 7, 8, 9, 5],
        [2, 3, 4, 0, 1, 7, 8, 9, 5, 6],
        [3, 4, 0, 1, 2, 8, 9, 5, 6, 7],
        [4, 0, 1, 2, 3, 9, 5, 6, 7, 8],
        [5, 9, 8, 7, 6, 4, 3, 2, 1, 0],
        [6, 5, 9, 8, 7, 0, 4, 3, 2, 1],
        [7, 6, 5, 9, 8, 1, 0, 4, 3, 2],
        [8, 7, 6, 5, 9, 2, 1, 0, 4, 3],
        [9, 8, 7, 6, 5, 3, 2, 1, 0, 4],
    ];

    let permutation_table = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        [1, 5, 7, 6, 2, 8, 3, 0, 9, 4],
        [5, 8, 0, 3, 7, 9, 6, 1, 4, 2],
        [8, 9, 1, 6, 0, 4, 3, 5, 2, 7],
        [9, 4, 5, 3, 1, 2, 6, 8, 7, 0],
        [4, 2, 8, 6, 5, 7, 3, 9, 0, 1],
        [2, 7, 9, 3, 8, 0, 6, 4, 1, 5],
        [7, 0, 4, 6, 9, 1, 3, 2, 5, 8],
    ];

    let inverse_table = [0, 4, 3, 2, 1, 5, 6, 7, 8, 9];

    let mut c = 0;
    let reversed_digits: Vec<usize> = aadhaar
        .chars()
        .rev()
        .map(|d| d.to_digit(10).unwrap() as usize)
        .collect();

    for (i, &digit) in reversed_digits.iter().enumerate() {
        c = multiplication_table[c][permutation_table[i % 8][digit]];
    }

    c == 0
}

// Session helper
fn get_saved_user() -> Option<User> {
    let storage = window().local_storage().ok()??;
    let user_json = storage.get_item("user").ok()??;
    serde_json::from_str(&user_json).ok()
}

fn save_user(user: &User) {
    if let Ok(storage) = window().local_storage() {
        if let Some(s) = storage {
            let _ = s.set_item("user", &serde_json::to_string(user).unwrap_or_default());
        }
    }
}

fn clear_user() {
    if let Ok(storage) = window().local_storage() {
        if let Some(s) = storage {
            let _ = s.remove_item("user");
        }
    }
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
                Err(_) => {
                    set_error.set(Some("Login failed. Check credentials.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    let on_mock_login = move |_| {
        let u = User { email: "admin@test.com".to_string(), uid: "mock_id".to_string() };
        save_user(&u);
        on_login.call(u);
    };

    view! {
        <div style="display: flex; justify-content: center; align-items: center; height: 100vh; flex-direction: column;">
            <div class="container" style="max-width: 400px; text-align: center;">
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
                <button on:click=on_mock_login style="margin-top: 10px; background-color: #6c757d; font-size: 0.8rem; padding: 5px 10px;">"DEBUG: Skip Login"</button>
            </div>
        </div>
    }
}

#[component]
fn DashboardHome() -> impl IntoView {
    view! {
        <div class="card">
            <h1>"Dashboard Overview"</h1>
            <p>"Welcome to your Lodge Management System. Select a category from the sidebar to manage your lodge."</p>
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
            <form on:submit=on_add_room style="display: flex; gap: 10px; align-items: flex-end; margin-bottom: 20px;">
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
                <button type="submit" style="margin-bottom: 1rem;">"Add Room"</button>
            </form>
            {move || if loading.get() { view! { <p>"Loading rooms..."</p> }.into_view() } else {
                view! {
                    <table style="width: 100%; border-collapse: collapse;">
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
    
    // Form signals
    let (name, set_name) = create_signal("".to_string());
    let (phone, set_phone) = create_signal("".to_string());
    let (email, set_email) = create_signal("".to_string());
    let (aadhaar, set_aadhaar) = create_signal("".to_string());
    let (photo, set_photo) = create_signal(None::<String>);
    let (id_card, set_id_card) = create_signal(None::<String>);

    // Camera signals
    let (camera_active, set_camera_active) = create_signal(false);
    let (capture_target, set_capture_target) = create_signal("photo"); // "photo" or "id"

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
        spawn_local(async move {
            let _ = start_camera("cam-preview".to_string()).await;
        });
    };

    let do_capture = move |_| {
        let data = take_snapshot("cam-preview".to_string());
        if capture_target.get() == "photo" {
            set_photo.set(Some(data));
        } else {
            set_id_card.set(Some(data));
        }
        stop_camera();
        set_camera_active.set(false);
    };

    let on_add_customer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        if !validate_aadhaar(&aadhaar.get()) {
            window().alert_with_message("Invalid Aadhaar Number! Please check the 12 digits.").ok();
            return;
        }

        let new_cust = Customer { 
            id: None, 
            full_name: name.get(), 
            phone: phone.get(), 
            email: email.get(),
            aadhaar: aadhaar.get(),
            photo_data: photo.get(),
            id_card_data: id_card.get(),
        };

        spawn_local(async move {
            let js_val = serde_wasm_bindgen::to_value(&new_cust).unwrap();
            match add_customer_js(js_val).await {
                Ok(_) => { 
                    set_name.set("".to_string()); 
                    set_phone.set("".to_string()); 
                    set_email.set("".to_string()); 
                    set_aadhaar.set("".to_string());
                    set_photo.set(None);
                    set_id_card.set(None);
                    load_customers(); 
                }
                Err(e) => logging::error!("Error adding customer: {:?}", e),
            }
        });
    };

    view! {
        <div class="card">
            <h1>"Customer Entry & Verification"</h1>
            
            <form on:submit=on_add_customer style="margin-bottom: 30px; background: #f9f9f9; padding: 20px; border-radius: 8px;">
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                    <div style="display: flex; flex-direction: column;">
                        <label>"Full Name"</label>
                        <input type="text" on:input=move |ev| set_name.set(event_target_value(&ev)) prop:value=name required />
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Phone Number"</label>
                        <input type="tel" on:input=move |ev| set_phone.set(event_target_value(&ev)) prop:value=phone required />
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Email Address"</label>
                        <input type="email" on:input=move |ev| set_email.set(event_target_value(&ev)) prop:value=email required />
                    </div>
                    <div style="display: flex; flex-direction: column;">
                        <label>"Aadhaar Number (12 Digits)"</label>
                        <input type="text" 
                            maxlength="12"
                            on:input=move |ev| set_aadhaar.set(event_target_value(&ev)) 
                            prop:value=aadhaar 
                            style=move || format!("border-color: {};", if validate_aadhaar(&aadhaar.get()) { "green" } else { "red" })
                            required 
                        />
                        <span style="font-size: 0.7rem; color: #666;">"Auto-verifying checksum..."</span>
                    </div>
                </div>

                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-top: 20px;">
                    <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;">
                        <p>"Customer Photo"</p>
                        {move || match photo.get() {
                            Some(data) => view! { <img src=data style="width: 100%; max-height: 150px; object-fit: contain;" /> }.into_view(),
                            None => view! { <div style="height: 100px; background: #eee; display: flex; align-items: center; justify-content: center;">"No Photo"</div> }.into_view()
                        }}
                        <button type="button" on:click=move |_| start_capture("photo") style="margin-top: 10px; font-size: 0.8rem;">"Take Photo"</button>
                    </div>
                    <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;">
                        <p>"Aadhaar Card Scan"</p>
                        {move || match id_card.get() {
                            Some(data) => view! { <img src=data style="width: 100%; max-height: 150px; object-fit: contain;" /> }.into_view(),
                            None => view! { <div style="height: 100px; background: #eee; display: flex; align-items: center; justify-content: center;">"No Scan"</div> }.into_view()
                        }}
                        <button type="button" on:click=move |_| start_capture("id") style="margin-top: 10px; font-size: 0.8rem;">"Scan ID"</button>
                    </div>
                </div>

                <button type="submit" style="width: 100%; margin-top: 20px; height: 50px; font-weight: bold;">"Verify & Save Customer"</button>
            </form>

            // Camera Modal
            {move || if camera_active.get() {
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 1000;">
                        <h2 style="color: white;">{move || format!("Capturing {}", capture_target.get())}</h2>
                        <video id="cam-preview" style="width: 80%; max-width: 600px; border: 2px solid white;"></video>
                        <div style="margin-top: 20px; display: flex; gap: 10px;">
                            <button on:click=do_capture style="background: green;">"CAPTURE"</button>
                            <button on:click=move |_| { stop_camera(); set_camera_active.set(false); } style="background: red;">"CANCEL"</button>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}

            <h3>"Verified Customer Directory"</h3>
            {move || if loading.get() { view! { <p>"Loading customers..."</p> }.into_view() } else {
                view! {
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr style="background-color: #f2f2f2; text-align: left;">
                                <th style="padding: 12px; border: 1px solid #ddd;">"Name"</th>
                                <th style="padding: 12px; border: 1px solid #ddd;">"Aadhaar"</th>
                                <th style="padding: 12px; border: 1px solid #ddd;">"Phone"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For each=move || customers.get() key=|c| c.id.clone().unwrap_or_default() children=|c| view! {
                                <tr>
                                    <td style="padding: 12px; border: 1px solid #ddd;">{c.full_name}</td>
                                    <td style="padding: 12px; border: 1px solid #ddd;">
                                        <span style="font-family: monospace;">{c.aadhaar}</span>
                                        " ✅"
                                    </td>
                                    <td style="padding: 12px; border: 1px solid #ddd;">{c.phone}</td>
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
fn DashboardLayout(user: User, on_logout: Callback<()>) -> impl IntoView {
    let handle_logout = move |_| {
        clear_user();
        spawn_local(async move {
            let _ = sign_out_user().await;
            on_logout.call(());
        });
    };

    view! {
        <div class="app-layout">
            <nav class="sidebar">
                <h2>"Lodge Manager"</h2>
                <A href="" class="nav-link" active_class="active" exact=true>"Overview"</A>
                <A href="rooms" class="nav-link" active_class="active">"Rooms"</A>
                <A href="customers" class="nav-link" active_class="active">"Customers"</A>
                
                <div style="margin-top: auto; padding-top: 1rem; border-top: 1px solid #444; font-size: 0.8rem;">
                    <p style="color: #bdc3c7; margin-bottom: 5px;">"Logged in as:"</p>
                    <p style="overflow: hidden; text-overflow: ellipsis; margin-bottom: 15px;">{user.email}</p>
                    <button on:click=handle_logout style="background-color: #e74c3c; width: 100%; padding: 8px;">"Logout"</button>
                </div>
            </nav>
            <main class="content">
                <Routes>
                    <Route path="" view=DashboardHome />
                    <Route path="rooms" view=Rooms />
                    <Route path="customers" view=Customers />
                </Routes>
            </main>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (user, set_user) = create_signal(get_saved_user());

    view! {
        <Router>
            <main>
                {move || match user.get() {
                    Some(u) => view! { <DashboardLayout user=u on_logout=Callback::new(move |_| set_user.set(None))/> }.into_view(),
                    None => view! { <Login on_login=Callback::new(move |u| set_user.set(Some(u)))/> }.into_view(),
                }}
            </main>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
