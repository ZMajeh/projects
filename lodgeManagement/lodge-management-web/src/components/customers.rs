use leptos::*;
use crate::models::{Customer, NewCustomer};
use crate::api::{get_customers_js, add_customer_js, update_customer_js, delete_customer_js, start_camera, take_snapshot, stop_camera, extract_aadhaar_js, readFileAsDataURL, manual_verify_aadhaar};
use crate::utils::{wait_for_bridge, validate_aadhaar_checksum, calculate_age};

pub async fn fetch_customers(search: String) -> Vec<Customer> {
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

#[component]
pub fn Customers() -> impl IntoView {
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

    let load_customers = move |q: String| {
        spawn_local(async move {
            set_loading.set(true);
            set_customers.set(fetch_customers(q).await);
            set_loading.set(false);
        });
    };

    create_effect(move |_| {
        let q = search_query.get();
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(500).await;
            if q == search_query.get_untracked() { load_customers(q); }
        });
    });

    let start_capture = move |target: &'static str| {
        set_capture_target.set(target);
        set_camera_active.set(true);
        spawn_local(async move { wait_for_bridge().await; let _ = start_camera("cam-preview".to_string()).await; });
    };

    let process_id_ocr = move |data: String| {
        set_ocr_loading.set(true);
        spawn_local(async move {
            wait_for_bridge().await;
            if let Ok(result_js) = extract_aadhaar_js(data).await {
                if let Some(obj) = js_sys::Object::try_from(&result_js) {
                    if let Ok(id_js) = js_sys::Reflect::get(obj, &JsValue::from_str("aadhaar")) {
                        if let Some(id) = id_js.as_string() { set_aadhaar.set(id); }
                    }
                    if let Ok(name_js) = js_sys::Reflect::get(obj, &JsValue::from_str("name")) {
                        if let Some(n) = name_js.as_string() { if name.get_untracked().is_empty() { set_name.set(n); } }
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

    let on_file_upload = move |ev: leptos::ev::Event, target: &'static str| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                spawn_local(async move {
                    wait_for_bridge().await;
                    if let Ok(data_js) = readFileAsDataURL(file).await {
                        if let Some(data) = data_js.as_string() {
                            if target == "photo" { set_photo.set(Some(data)); } 
                            else { set_id_card.set(Some(data.clone())); process_id_ocr(data); }
                        }
                    }
                });
            }
        }
    };

    let on_verify_trigger = move |num: String, id: Option<String>| {
        set_manual_verify_target_id.set(id);
        spawn_local(async move {
            wait_for_bridge().await;
            let _ = manual_verify_aadhaar(num).await;
            set_show_manual_verify.set(true);
        });
    };

    let on_manual_confirm = move |is_valid: bool| {
        let target_id = manual_verify_target_id.get();
        set_show_manual_verify.set(false);
        if is_valid {
            if let Some(id) = target_id {
                spawn_local(async move {
                    wait_for_bridge().await;
                    let patch = serde_json::json!({ "verified": true });
                    let js_val = serde_wasm_bindgen::to_value(&patch).unwrap();
                    let _ = update_customer_js(id, js_val).await;
                    load_customers(search_query.get_untracked());
                });
            } else { set_is_verified.set(true); }
        }
    };

    let on_add_customer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if !is_verified.get() && editing_id.get().is_none() { window().alert_with_message("Please verify Aadhaar first!").ok(); return; }
        let cust_data = NewCustomer { 
            full_name: name.get(), phone: phone.get(), email: email.get(), aadhaar: aadhaar.get(), 
            age: Some(age.get()), gender: Some(gender.get()), photo_data: photo.get(), id_card_data: id_card.get(),
            verified: is_verified.get()
        };
        spawn_local(async move {
            wait_for_bridge().await;
            let js_val = serde_wasm_bindgen::to_value(&cust_data).unwrap();
            if let Some(id) = editing_id.get() { let _ = update_customer_js(id, js_val).await; } 
            else { let _ = add_customer_js(js_val).await; }
            set_editing_id.set(None);
            set_name.set("".to_string()); set_phone.set("".to_string()); set_email.set("".to_string()); 
            set_aadhaar.set("".to_string()); set_age.set("".to_string()); set_gender.set("Male".to_string());
            set_photo.set(None); set_id_card.set(None); set_is_verified.set(false);
            load_customers(search_query.get_untracked());
        });
    };

    let on_edit = move |c: Customer| {
        set_editing_id.set(c.id.clone());
        set_name.set(c.full_name); set_phone.set(c.phone); set_email.set(c.email);
        set_aadhaar.set(c.aadhaar); set_age.set(c.age.unwrap_or_default());
        set_gender.set(c.gender.unwrap_or_else(|| "Male".to_string()));
        set_photo.set(c.photo_data); set_id_card.set(c.id_card_data);
        set_is_verified.set(c.verified); window().scroll_to_with_x_and_y(0.0, 0.0);
    };

    let on_delete = move |id: String| {
        if window().confirm_with_message("Delete?").unwrap_or(false) {
            spawn_local(async move { wait_for_bridge().await; let _ = delete_customer_js(id).await; load_customers(search_query.get_untracked()); });
        }
    };

    view! {
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h1>"Customers"</h1>
                {move || if editing_id.get().is_some() { view! { <button on:click=move |_| set_editing_id.set(None) style="background:#6c757d;">"Cancel"</button> }.into_view() } else { view! {}.into_view() }}
            </div>
            <form on:submit=on_add_customer class="card" style="background: #f9f9f9;">
                <div class="grid-form">
                    <div style="display: flex; flex-direction: column;"><label>"Full Name"</label><input type="text" on:input=move |ev| set_name.set(event_target_value(&ev)) prop:value=name required /></div>
                    <div style="display: flex; flex-direction: column;"><label>"Phone"</label><input type="tel" on:input=move |ev| set_phone.set(event_target_value(&ev)) prop:value=phone required /></div>
                    <div style="display: flex; flex-direction: column;"><label>"Age"</label><input type="number" on:input=move |ev| set_age.set(event_target_value(&ev)) prop:value=age required /></div>
                    <div style="display: flex; flex-direction: column;"><label>"Gender"</label><select on:change=move |ev| set_gender.set(event_target_value(&ev)) prop:value=gender><option value="Male">"Male"</option><option value="Female">"Female"</option><option value="Other">"Other"</option></select></div>
                    <div style="display: flex; flex-direction: column; grid-column: 1 / -1;"><label>"Aadhaar"</label><div style="display: flex; gap: 5px;"><input type="text" maxlength="12" on:input=move |ev| { set_aadhaar.set(event_target_value(&ev)); set_is_verified.set(false); } prop:value=aadhaar style=move || format!("border-color: {};", if is_verified.get() { "green" } else { "#ddd" }) required /><button type="button" on:click=move |_| on_verify_trigger(aadhaar.get(), None) disabled=ocr_loading>"Verify"</button></div></div>
                </div>
                <div class="grid-form" style="margin-top: 20px;">
                    <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;"><p>"Photo"</p>{move || photo.get().map(|d| view! { <img src=d style="width: 100%; max-height: 100px;" /> })}<div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;"><button type="button" on:click=move |_| start_capture("photo") style="font-size: 0.8rem;">"Camera"</button><input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "photo") style="font-size: 0.7rem;" /></div></div>
                    <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;"><p>"ID Scan"</p>{move || id_card.get().map(|d| view! { <img src=d style="width: 100%; max-height: 100px;" /> })}<div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;"><button type="button" on:click=move |_| start_capture("id") style="font-size: 0.8rem;">"Camera"</button><input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "id") style="font-size: 0.7rem;" /></div></div>
                </div>
                <button type="submit" style="width: 100%; margin-top: 20px; background-color: #27ae60;">{move || if editing_id.get().is_some() { "Update Customer" } else { "Save Verified Customer" }}</button>
            </form>
            <div style="margin: 2rem 0;"><input type="text" placeholder="Search name/Aadhaar/phone..." on:input=move |ev| set_search_query.set(event_target_value(&ev)) style="padding: 1rem; font-size: 1.1rem; border: 2px solid var(--primary);" /></div>
            <h3>"Directory"</h3>
            {move || if loading.get() { view! { <p>"Loading..."</p> }.into_view() } else { view! { <table><thead><tr style="background-color: #f2f2f2; text-align: left;"><th>"Name"</th><th>"Aadhaar"</th><th>"Contact"</th><th>"Status"</th><th>"Actions"</th></tr></thead><tbody><For each=move || customers.get() key=|c| c.id.clone().unwrap_or_default() children=move |c| { let c_cloned = c.clone(); let id_cloned = c.id.clone().unwrap_or_default(); let num_cloned = c.aadhaar.clone(); view! { <tr><td><strong>{c.full_name.clone()}</strong><br/><small>{c.age.clone().unwrap_or_else(|| "??".to_string())} {c.gender.clone().unwrap_or_else(|| "??".to_string())}</small></td><td>{c.aadhaar.clone()}</td><td>{c.phone.clone()}<br/><small style="color: #666;">{c.email.clone()}</small></td><td>{if c.verified { view! { <span style="color: green; font-weight: bold;">"✅ Verified"</span> }.into_view() } else { let i_cloned = id_cloned.clone(); let n_cloned = num_cloned.clone(); view! { <button on:click=move |_| on_verify_trigger(n_cloned.clone(), Some(i_cloned.clone())) style="padding: 4px 8px; font-size: 0.7rem; background: #f39c12;">"Verify Now"</button> }.into_view() }}</td><td><button on:click=move |_| on_edit(c_cloned.clone()) style="padding: 5px 10px; margin-right: 5px; font-size: 0.8rem; background: #3498db;">"Edit"</button><button on:click=move |_| on_delete(id_cloned.clone()) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Del"</button></td></tr> } } /></tbody></table> }.into_view() }}{move || if show_manual_verify.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 3000;"><div class="card" style="max-width: 400px; text-align: center; padding: 2rem;"><h3>"Confirm Aadhaar"</h3><p>"Official UIDAI site opened. ID copied."</p><div style="display: flex; gap: 10px; margin-top: 20px;"><button on:click=move |_| on_manual_confirm(true) style="background: green; flex: 1;">"YES"</button><button on:click=move |_| on_manual_confirm(false) style="background: red; flex: 1;">"NO"</button></div></div></div> }.into_view() } else { view! {}.into_view() }}{move || if camera_active.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.9); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 2000; padding: 1rem;"><video id="cam-preview" style="width: 100%; max-width: 500px; border: 2px solid white;"></video><div style="margin-top: 20px; display: flex; gap: 10px;"><button on:click=move |_: leptos::ev::MouseEvent| { spawn_local(async move { wait_for_bridge().await; if let Ok(data_js) = take_snapshot("cam-preview".to_string()).await { if let Some(data) = data_js.as_string() { if capture_target.get() == "photo" { set_photo.set(Some(data)); } else { set_id_card.set(Some(data.clone())); process_id_ocr(data); } } } let _ = stop_camera().await; set_camera_active.set(false); }); } style="background: green;">"CAPTURE"</button><button on:click=move |_: leptos::ev::MouseEvent| { spawn_local(async move { let _ = stop_camera().await; set_camera_active.set(false); }); } style="background: red;">"CLOSE"</button></div></div> }.into_view() } else { view! {}.into_view() }}
        </div>
    }
}
