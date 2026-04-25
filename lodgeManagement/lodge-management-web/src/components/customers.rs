use leptos::*;
use wasm_bindgen::prelude::*;
use crate::models::{Customer, NewCustomer};
use crate::api::{get_customers_js, add_customer_js, update_customer_js, delete_customer_js, start_camera, take_snapshot, stop_camera, extract_aadhaar_js, read_file_as_data_url, manual_verify_aadhaar};
use crate::utils::{wait_for_bridge, calculate_age};

#[component]
pub fn CustomerForm(
    editing_id: Memo<Option<String>>,
    on_success: Callback<()>,
    initial_data: Option<Customer>,
) -> impl IntoView {
    let (name, set_name) = create_signal(initial_data.as_ref().map(|c| c.full_name.clone()).unwrap_or_default());
    let (phone, set_phone) = create_signal(initial_data.as_ref().map(|c| c.phone.clone()).unwrap_or_default());
    let (aadhaar, set_aadhaar) = create_signal(initial_data.as_ref().map(|c| c.aadhaar.clone()).unwrap_or_default());
    let (age, set_age) = create_signal(initial_data.as_ref().and_then(|c| c.age.clone()).unwrap_or_default());
    let (gender, set_gender) = create_signal(initial_data.as_ref().and_then(|c| c.gender.clone()).unwrap_or("Male".to_string()));
    
    let (photo, set_photo) = create_signal(initial_data.as_ref().and_then(|c| c.photo_url.clone()));
    let (id_card, set_id_card) = create_signal(initial_data.as_ref().and_then(|c| c.id_card_url.clone()));
    let (id_card_back, set_id_card_back) = create_signal(initial_data.as_ref().and_then(|c| c.id_card_back_url.clone()));
    
    let (camera_active, set_camera_active) = create_signal(false);
    let (capture_target, set_capture_target) = create_signal("photo");
    let (ocr_loading, set_ocr_loading) = create_signal(false);
    let (is_verified, set_is_verified) = create_signal(initial_data.as_ref().map(|c| c.verified).unwrap_or(false));
    let (show_manual_verify, set_show_manual_verify) = create_signal(false);
    let (is_saving, set_is_saving) = create_signal(false);
    let (upload_status, set_upload_status) = create_signal("".to_string());

    let start_capture = move |target: &'static str| {
        set_capture_target.set(target);
        set_camera_active.set(true);
        spawn_local(async move { wait_for_bridge().await; let _ = start_camera("cam-preview-modal".to_string()).await; });
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
                    if let Ok(data_js) = read_file_as_data_url(file).await {
                        if let Some(data) = data_js.as_string() {
                            if target == "photo" { set_photo.set(Some(data)); } 
                            else if target == "id" { set_id_card.set(Some(data.clone())); process_id_ocr(data); }
                            else if target == "id_back" { set_id_card_back.set(Some(data)); }
                        }
                    }
                });
            }
        }
    };

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if !is_verified.get() && editing_id.get().is_none() { window().alert_with_message("Please verify Aadhaar first!").ok(); return; }
        
        if !crate::api::is_drive_authorized() && (photo.get_untracked().map(|d| d.starts_with("data:")).unwrap_or(false) || id_card.get_untracked().map(|d| d.starts_with("data:")).unwrap_or(false) || id_card_back.get_untracked().map(|d| d.starts_with("data:")).unwrap_or(false)) {
             window().alert_with_message("Google Drive not authorized! Please sign in with Google in the sidebar first.").ok();
             return;
        }

        let name_val = name.get();
        let phone_val = phone.get();
        let aadhaar_val = aadhaar.get();
        let age_val = Some(age.get());
        let gender_val = Some(gender.get());
        let photo_val = photo.get();
        let id_card_val = id_card.get();
        let id_card_back_val = id_card_back.get();
        let verified_val = is_verified.get();
        let editing_id_val = editing_id.get();

        set_is_saving.set(true);

        let initial_data_cloned = initial_data.clone();

        spawn_local(async move {
            wait_for_bridge().await;
            
            // 1. Calculate how many uploads are needed
            let mut total_uploads = 0;
            if photo_val.as_ref().map(|d| d.starts_with("data:")).unwrap_or(false) { total_uploads += 1; }
            if id_card_val.as_ref().map(|d| d.starts_with("data:")).unwrap_or(false) { total_uploads += 1; }
            if id_card_back_val.as_ref().map(|d| d.starts_with("data:")).unwrap_or(false) { total_uploads += 1; }
            
            let mut current_upload = 0;
            
            // Searchable slug
            let search_slug = format!("{}_{}_{}_{}", 
                name_val.replace(" ", "-"), 
                phone_val,
                age_val.as_deref().unwrap_or("0"),
                aadhaar_val
            );

            let get_id = |url: &Option<String>| {
                url.as_ref().and_then(|u| {
                    if u.contains("drive.google.com") {
                        u.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).map(|s| s.to_string())
                    } else { None }
                })
            };

            let photo_url = if let Some(d) = photo_val {
                if d.starts_with("data:") {
                    current_upload += 1;
                    set_upload_status.set(format!("Uploading Photo ({}/{})", current_upload, total_uploads));
                    if let Some(old_id) = get_id(&initial_data_cloned.as_ref().and_then(|c| c.photo_url.clone())) {
                        let _ = crate::api::delete_file_from_drive(old_id).await;
                    }
                    let filename = format!("PHOTO_{}.jpg", search_slug);
                    crate::api::upload_image_to_drive(d, filename).await.ok().and_then(|v| v.as_string())
                } else { Some(d) }
            } else { None };

            let id_url = if let Some(d) = id_card_val {
                if d.starts_with("data:") {
                    current_upload += 1;
                    set_upload_status.set(format!("Uploading ID Front ({}/{})", current_upload, total_uploads));
                    if let Some(old_id) = get_id(&initial_data_cloned.as_ref().and_then(|c| c.id_card_url.clone())) {
                        let _ = crate::api::delete_file_from_drive(old_id).await;
                    }
                    let filename = format!("ID_FRONT_{}.jpg", search_slug);
                    crate::api::upload_image_to_drive(d, filename).await.ok().and_then(|v| v.as_string())
                } else { Some(d) }
            } else { None };

            let id_back_url = if let Some(d) = id_card_back_val {
                if d.starts_with("data:") {
                    current_upload += 1;
                    set_upload_status.set(format!("Uploading ID Back ({}/{})", current_upload, total_uploads));
                    if let Some(old_id) = get_id(&initial_data_cloned.as_ref().and_then(|c| c.id_card_back_url.clone())) {
                        let _ = crate::api::delete_file_from_drive(old_id).await;
                    }
                    let filename = format!("ID_BACK_{}.jpg", search_slug);
                    crate::api::upload_image_to_drive(d, filename).await.ok().and_then(|v| v.as_string())
                } else { Some(d) }
            } else { None };

            set_upload_status.set("Saving record...".to_string());

            let cust_data = NewCustomer { 
                full_name: name_val, phone: phone_val, email: "".to_string(), aadhaar: aadhaar_val, 
                age: age_val, gender: gender_val, photo_url, 
                id_card_url: id_url, id_card_back_url: id_back_url,
                verified: verified_val
            };

            let js_val = serde_wasm_bindgen::to_value(&cust_data).unwrap();
            if let Some(id) = editing_id_val { let _ = update_customer_js(id, js_val).await; } 
            else { let _ = add_customer_js(js_val).await; }
            
            set_is_saving.set(false);
            on_success.call(());
        });
    };

    view! {
        <form on:submit=handle_submit class="card" style="background: #f9f9f9; text-align: left;">
            <div class="grid-form">
                <div style="display: flex; flex-direction: column;"><label>"Full Name"</label><input type="text" on:input=move |ev| set_name.set(event_target_value(&ev)) prop:value=name required /></div>
                <div style="display: flex; flex-direction: column;"><label>"Phone"</label><input type="tel" on:input=move |ev| set_phone.set(event_target_value(&ev)) prop:value=phone required /></div>
                <div style="display: flex; flex-direction: column;"><label>"Age"</label><input type="number" on:input=move |ev| set_age.set(event_target_value(&ev)) prop:value=age required /></div>
                <div style="display: flex; flex-direction: column;"><label>"Gender"</label><select on:change=move |ev| set_gender.set(event_target_value(&ev)) prop:value=gender><option value="Male">"Male"</option><option value="Female">"Female"</option><option value="Other">"Other"</option></select></div>
                <div style="display: flex; flex-direction: column; grid-column: 1 / -1;"><label>"Aadhaar"</label><div style="display: flex; gap: 5px;"><input type="text" maxlength="12" on:input=move |ev| { set_aadhaar.set(event_target_value(&ev)); set_is_verified.set(false); } prop:value=aadhaar required /><button type="button" on:click=move |_| { let val = aadhaar.get_untracked(); spawn_local(async move { wait_for_bridge().await; let _ = manual_verify_aadhaar(val).await; set_show_manual_verify.set(true); }); } disabled=ocr_loading>"Verify"</button></div></div>
            </div>
            <div class="grid-form" style="margin-top: 20px;">
                <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;"><p>"Photo"</p>{move || photo.get().map(|d| {
                    if d.contains("drive.google.com") {
                        let id = d.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).unwrap_or("").to_string();
                        let url_res = create_resource(move || id.clone(), |fid| async move {
                            crate::api::get_drive_thumbnail(fid).await.ok().and_then(|v| v.as_string())
                        });
                        view! { 
                            <Suspense fallback=move || view! { <p>"..."</p> }>
                                {move || url_res.get().flatten().map(|u| view! { <img src=u style="width: 100%; max-height: 80px;" /> })}
                            </Suspense>
                        }.into_view()
                    } else {
                        view! { <img src=d style="width: 100%; max-height: 80px;" /> }.into_view()
                    }
                })}<div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;"><button type="button" on:click=move |_| start_capture("photo") style="font-size: 0.7rem; padding: 5px;">"Camera"</button><input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "photo") style="font-size: 0.6rem;" /></div></div>
                <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;"><p>"ID Front"</p>{move || id_card.get().map(|d| {
                    if d.contains("drive.google.com") {
                        let id = d.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).unwrap_or("").to_string();
                        let url_res = create_resource(move || id.clone(), |fid| async move {
                            crate::api::get_drive_thumbnail(fid).await.ok().and_then(|v| v.as_string())
                        });
                        view! { 
                            <Suspense fallback=move || view! { <p>"..."</p> }>
                                {move || url_res.get().flatten().map(|u| view! { <img src=u style="width: 100%; max-height: 80px;" /> })}
                            </Suspense>
                        }.into_view()
                    } else {
                        view! { <img src=d style="width: 100%; max-height: 80px;" /> }.into_view()
                    }
                })}<div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;"><button type="button" on:click=move |_| start_capture("id") style="font-size: 0.7rem; padding: 5px;">"Camera"</button><input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "id") style="font-size: 0.6rem;" /></div></div>
                <div style="text-align: center; border: 1px dashed #ccc; padding: 10px;"><p>"ID Back"</p>{move || id_card_back.get().map(|d| {
                    if d.contains("drive.google.com") {
                        let id = d.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).unwrap_or("").to_string();
                        let url_res = create_resource(move || id.clone(), |fid| async move {
                            crate::api::get_drive_thumbnail(fid).await.ok().and_then(|v| v.as_string())
                        });
                        view! { 
                            <Suspense fallback=move || view! { <p>"..."</p> }>
                                {move || url_res.get().flatten().map(|u| view! { <img src=u style="width: 100%; max-height: 80px;" /> })}
                            </Suspense>
                        }.into_view()
                    } else {
                        view! { <img src=d style="width: 100%; max-height: 80px;" /> }.into_view()
                    }
                })}<div style="display: flex; flex-direction: column; gap: 5px; margin-top: 5px;"><button type="button" on:click=move |_| start_capture("id_back") style="font-size: 0.7rem; padding: 5px;">"Camera"</button><input type="file" accept="image/*" on:change=move |ev| on_file_upload(ev, "id_back") style="font-size: 0.6rem;" /></div></div>
            </div>
            <button type="submit" style="width: 100%; margin-top: 20px; background-color: #27ae60;" disabled=is_saving>
                {move || if is_saving.get() { upload_status.get() } else if editing_id.get().is_some() { "Update".to_string() } else { "Save Verified Guest".to_string() }}
            </button>
            
            {move || if show_manual_verify.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 4000;"><div class="card" style="max-width: 300px; text-align: center; padding: 1.5rem;"><h3>"Verified?"</h3><div style="display: flex; gap: 10px; margin-top: 20px;"><button type="button" on:click=move |_| { set_is_verified.set(true); set_show_manual_verify.set(false); } style="background: green; flex: 1;">"YES"</button><button type="button" on:click=move |_| set_show_manual_verify.set(false) style="background: red; flex: 1;">"NO"</button></div></div></div> }.into_view() } else { view! {}.into_view() }}
            {move || if camera_active.get() { view! { <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.9); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 4000; padding: 1rem;"><video id="cam-preview-modal" style="width: 100%; max-width: 400px; border: 2px solid white;"></video><div style="margin-top: 20px; display: flex; gap: 10px;"><button type="button" on:click=move |_| { spawn_local(async move { wait_for_bridge().await; if let Ok(data_js) = take_snapshot("cam-preview-modal".to_string()).await { if let Some(data) = data_js.as_string() { if capture_target.get() == "photo" { set_photo.set(Some(data)); } else if capture_target.get() == "id" { set_id_card.set(Some(data.clone())); process_id_ocr(data); } else if capture_target.get() == "id_back" { set_id_card_back.set(Some(data)); } } } let _ = stop_camera().await; set_camera_active.set(false); }); } style="background: green;">"CAPTURE"</button><button type="button" on:click=move |_| { spawn_local(async move { let _ = stop_camera().await; set_camera_active.set(false); }); } style="background: red;">"CLOSE"</button></div></div> }.into_view() } else { view! {}.into_view() }}
        </form>
    }
}

pub async fn fetch_customers(search: String) -> Vec<Customer> {
    wait_for_bridge().await;
    match get_customers_js(search).await {
        Ok(js_val) => serde_wasm_bindgen::from_value::<Vec<Customer>>(js_val).unwrap_or_default(),
        Err(_) => vec![],
    }
}

#[component]
pub fn Customers() -> impl IntoView {
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (loading, set_loading) = create_signal(true);
    let (search_query, set_search_query) = create_signal("".to_string());
    let (editing_id, set_editing_id) = create_signal(None::<String>);
    let (editing_data, set_editing_data) = create_signal(None::<Customer>);
    let (confirm_delete_id, set_confirm_delete_id) = create_signal(None::<String>);

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

    let on_edit = move |c: Customer| {
        set_editing_id.set(c.id.clone());
        set_editing_data.set(Some(c));
        window().scroll_to_with_x_and_y(0.0, 0.0);
    };

    let on_delete_final = move |c: Customer| {
        let id = c.id.clone().unwrap_or_default();
        spawn_local(async move {
            wait_for_bridge().await;

            // Helper to extract File ID from Drive URL
            let get_id = |url: &Option<String>| {
                url.as_ref().and_then(|u| {
                    if u.contains("drive.google.com") {
                        u.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).map(|s| s.to_string())
                    } else { None }
                })
            };

            // Delete associated files from Drive
            if let Some(fid) = get_id(&c.photo_url) { let _ = crate::api::delete_file_from_drive(fid).await; }
            if let Some(fid) = get_id(&c.id_card_url) { let _ = crate::api::delete_file_from_drive(fid).await; }
            if let Some(fid) = get_id(&c.id_card_back_url) { let _ = crate::api::delete_file_from_drive(fid).await; }

            // Delete from Firestore
            let _ = delete_customer_js(id).await;
            set_confirm_delete_id.set(None);
            load_customers(search_query.get_untracked());
        });
    };

    view! {
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h1>"Customers"</h1>
                {move || if editing_id.get().is_some() { view! { <button on:click=move |_| { set_editing_id.set(None); set_editing_data.set(None); } style="background:#6c757d;">"Cancel Edit"</button> }.into_view() } else { view! {}.into_view() }}
            </div>
            
            {move || {
                let e_id = editing_id.get();
                let e_data = editing_data.get();
                view! {
                    <div id=e_id.clone().unwrap_or_else(|| "new-form".to_string())>
                        <CustomerForm 
                            editing_id=create_memo(move |_| e_id.clone()) 
                            initial_data=e_data
                            on_success=Callback::new(move |_| { set_editing_id.set(None); set_editing_data.set(None); load_customers(search_query.get_untracked()); }) 
                        />
                    </div>
                }
            }}

            <div style="margin: 2rem 0;"><input type="text" placeholder="Search..." on:input=move |ev| set_search_query.set(event_target_value(&ev)) /></div>
            <h3>"Guest Directory"</h3>
            {move || if loading.get() { view! { <p>"Loading..."</p> }.into_view() } else { view! { 
                <table>
                    <thead><tr><th>"Name"</th><th>"Aadhaar"</th><th>"Status"</th><th>"Actions"</th></tr></thead>
                    <tbody>
                        <For each=move || customers.get() key=|c| c.id.clone().unwrap_or_default() children=move |c| { 
                            let c_cloned = c.clone(); 
                            let id_cloned = c.id.clone().unwrap_or_default(); 
                            let id_c = id_cloned.clone();
                            
                            view! { 
                                <tr>
                                    <td><strong>{c.full_name.clone()}</strong></td>
                                    <td>{c.aadhaar.clone()}</td>
                                    <td>{if c.verified { "✅" } else { "⚠️" }}</td>
                                    <td>
                                        {move || if confirm_delete_id.get() == Some(id_c.clone()) {
                                            let c_final = c_cloned.clone();
                                            view! {
                                                <div style="display: flex; gap: 5px; align-items: center;">
                                                    <span style="font-size: 0.7rem; color: red;">"Sure?"</span>
                                                    <button on:click=move |_| on_delete_final(c_final.clone()) style="padding: 2px 8px; font-size: 0.7rem; background: #e74c3c;">"YES"</button>
                                                    <button on:click=move |_| set_confirm_delete_id.set(None) style="padding: 2px 8px; font-size: 0.7rem; background: #6c757d;">"NO"</button>
                                                </div>
                                            }.into_view()
                                        } else {
                                            let c_edit = c_cloned.clone();
                                            let id_del = id_c.clone();
                                            view! {
                                                <div style="display: flex; gap: 5px;">
                                                    <button on:click=move |_| on_edit(c_edit.clone()) style="padding: 5px 10px; font-size: 0.8rem; background: #3498db;">"Edit"</button>
                                                    <button on:click=move |_| set_confirm_delete_id.set(Some(id_del.clone())) style="padding: 5px 10px; font-size: 0.8rem; background: #e74c3c;">"Del"</button>
                                                </div>
                                            }.into_view()
                                        }}
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
