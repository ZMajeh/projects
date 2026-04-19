use leptos::*;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub uid: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = addTestLodge)]
    async fn add_test_lodge() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = loginUser)]
    async fn login_user(email: String, pass: String) -> Result<JsValue, JsValue>;
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
                        on_login.call(user);
                    }
                }
                Err(_) => {
                    set_error.set(Some("Login failed. Please check credentials or Firebase setup.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container" style="max-width: 400px;">
            <h2>"Lodge Management Login"</h2>
            <form on:submit=on_submit>
                <div style="margin-bottom: 15px; text-align: left;">
                    <label style="display: block; margin-bottom: 5px;">"Email"</label>
                    <input 
                        type="email" 
                        style="width: 100%; padding: 8px; box-sizing: border-box;"
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                        prop:value=email
                        required
                    />
                </div>
                <div style="margin-bottom: 15px; text-align: left;">
                    <label style="display: block; margin-bottom: 5px;">"Password"</label>
                    <input 
                        type="password" 
                        style="width: 100%; padding: 8px; box-sizing: border-box;"
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        prop:value=password
                        required
                    />
                </div>
                {move || error.get().map(|err| view! { <p style="color: red; font-size: 0.9rem;">{err}</p> })}
                <button type="submit" style="width: 100%;" disabled=loading>
                    {move || if loading.get() { "Logging in..." } else { "Login" }}
                </button>
            </form>
        </div>
    }
}

#[component]
fn Dashboard(user: User) -> impl IntoView {
    let (status, set_status) = create_signal("Ready".to_string());
    
    let on_test_click = move |_| {
        spawn_local(async move {
            set_status.set("Writing to database...".to_string());
            match add_test_lodge().await {
                Ok(id) => {
                    let id_str = id.as_string().unwrap_or_else(|| "unknown".to_string());
                    set_status.set(format!("Success! New Lodge ID: {}", id_str));
                }
                Err(_) => set_status.set("Database Error!".to_string()),
            }
        });
    };

    view! {
        <div class="container">
            <h1>"Lodge Dashboard"</h1>
            <p>"Welcome, " {user.email}</p>
            <hr/>
            <div style="display: flex; gap: 20px; justify-content: center; margin-top: 20px;">
                <div style="padding: 20px; border: 1px solid #ccc; border-radius: 8px;">
                    <h3>"Quick Actions"</h3>
                    <button on:click=on_test_click>"Test Database Write"</button>
                    <p style="font-size: 0.8rem;">{move || status.get()}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (user, set_user) = create_signal(None::<User>);

    view! {
        <main>
            {move || match user.get() {
                Some(u) => view! { <Dashboard user=u/> }.into_view(),
                None => view! { <Login on_login=Callback::new(move |u| set_user.set(Some(u)))/> }.into_view(),
            }}
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
