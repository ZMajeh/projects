use leptos::*;
use crate::api::{get_whitelisted_users, add_user_to_whitelist, delete_user_from_whitelist};
use crate::utils::wait_for_bridge;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WhitelistedUser {
    pub email: String,
    pub role: String,
}

#[component]
pub fn ManageUsers() -> impl IntoView {
    let (users, set_users) = create_signal(Vec::<WhitelistedUser>::new());
    let (new_email, set_new_email) = create_signal("".to_string());
    let (new_role, set_new_role) = create_signal("Staff".to_string());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);

    let fetch_users = move || {
        spawn_local(async move {
            wait_for_bridge().await;
            logging::log!("Staff Management: Fetching users...");
            match get_whitelisted_users().await {
                Ok(js_val) => {
                    logging::log!("Staff Management: Received JS value, attempting to deserialize...");
                    match serde_wasm_bindgen::from_value::<Vec<WhitelistedUser>>(js_val) {
                        Ok(users_vec) => {
                            logging::log!("Staff Management: Successfully fetched {} users.", users_vec.len());
                            set_users.set(users_vec);
                        }
                        Err(e) => {
                            logging::error!("Staff Management: Deserialization failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    logging::error!("Staff Management: JS Bridge error: {:?}", e);
                }
            }
        });
    };

    // Initial fetch
    create_effect(move |_| fetch_users());

    let handle_add_user = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let email = new_email.get();
        let role = new_role.get();
        
        if email.is_empty() { return; }
        
        set_loading.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            wait_for_bridge().await;
            match add_user_to_whitelist(email, role).await {
                Ok(_) => {
                    set_new_email.set("".to_string());
                    fetch_users();
                }
                Err(_) => set_error.set(Some("Failed to add user. Check permissions.".to_string())),
            }
            set_loading.set(false);
        });
    };

    let handle_delete = move |email: String| {
        spawn_local(async move {
            wait_for_bridge().await;
            let _ = delete_user_from_whitelist(email).await;
            fetch_users();
        });
    };

    view! {
        <div class="container">
            <h2 style="color: var(--primary); margin-bottom: 1.5rem;">"Manage Staff Access"</h2>
            
            <div class="card" style="border-left: 5px solid #f1c40f; background: #fffde7; padding: 1.5rem; margin-bottom: 2rem;">
                <h3 style="margin-top: 0; color: #d35400;">"Onboarding Guidelines"</h3>
                <p>"To successfully add a new staff member, follow these steps in order:"</p>
                <ol style="line-height: 1.6;">
                    <li>
                        <strong>"Share Drive Folder: "</strong>
                        <a href="https://drive.google.com/drive/folders/1aCEzUmQwvbNg-J0sTyUaaftxjyOO7dQc" target="_blank" style="color: var(--primary); font-weight: bold;">"Open Photos Folder"</a>
                        " - Add their email as 'Viewer' or 'Editor'."
                    </li>
                    <li>
                        <strong>"Add to Whitelist: "</strong>
                        "Use the form below to add their email to the database."
                    </li>
                    <li>
                        <strong>"Register in Google Cloud: "</strong>
                        <a href="https://console.cloud.google.com/auth/audience?project=lodge-management-a4cc9" target="_blank" style="color: var(--primary); font-weight: bold;">"Open OAuth Audience Page"</a>
                        " - Add them as a 'Test User'."
                    </li>
                </ol>
                <p style="font-size: 0.85rem; color: #7f8c8d; margin-bottom: 0;">
                    <em>"Note: Users will see an 'Unauthorized' error if any of these steps are missed."</em>
                </p>
            </div>

            <div class="card">
                <h3>"Add New User"</h3>
                <form on:submit=handle_add_user class="grid-form">
                    <div>
                        <label>"Gmail Address"</label>
                        <input type="email" 
                            on:input=move |ev| set_new_email.set(event_target_value(&ev)) 
                            prop:value=new_email 
                            placeholder="example@gmail.com" required />
                    </div>
                    <div>
                        <label>"Role"</label>
                        <select on:change=move |ev| set_new_role.set(event_target_value(&ev)) prop:value=new_role>
                            <option value="Staff">"Staff"</option>
                            <option value="Admin">"Admin"</option>
                        </select>
                    </div>
                    <div style="grid-column: 1 / -1; margin-top: 10px;">
                        <button type="submit" style="width: 100%;" disabled=loading>
                            {move || if loading.get() { "Adding..." } else { "Add User to Whitelist" }}
                        </button>
                    </div>
                </form>
                {move || error.get().map(|err| view! { <p style="color: #e74c3c; margin-top: 10px;">{err}</p> })}
            </div>

            <div class="card" style="margin-top: 2rem;">
                <h3>"Current Whitelisted Users"</h3>
                <table>
                    <thead>
                        <tr>
                            <th>"Email"</th>
                            <th>"Role"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || users.get()
                            key=|u| u.email.clone()
                            children=move |u| {
                                let email = u.email.clone();
                                let role = u.role.clone();
                                view! {
                                    <tr>
                                        <td>{u.email}</td>
                                        <td>
                                            <span style=format!("padding: 2px 8px; border-radius: 4px; font-size: 0.8rem; font-weight: bold; background: {}; color: white;", 
                                                if role == "Admin" { "#9b59b6" } else { "#3498db" })
                                            >
                                                {role.clone()}
                                            </span>
                                        </td>
                                        <td>
                                            {if role != "Admin" {
                                                view! {
                                                    <button 
                                                        on:click=move |_| handle_delete(email.clone())
                                                        style="background: #e74c3c; padding: 5px 10px; font-size: 0.8rem;"
                                                    >
                                                        "Remove"
                                                    </button>
                                                }.into_view()
                                            } else {
                                                view! { <span style="color: #95a5a6; font-style: italic; font-size: 0.8rem;">"Protected"</span> }.into_view()
                                            }}
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </div>
    }
}
