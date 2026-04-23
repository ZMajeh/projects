use leptos::*;
use leptos_router::use_navigate;
use crate::models::User;
use crate::api::login_user;
use crate::utils::{save_user, wait_for_bridge};

#[component]
pub fn Login(on_login: Callback<User>) -> impl IntoView {
    let (email, set_email) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);
        let email_val = email.get();
        let pass_val = password.get();
        let navigate = navigate.clone();
        spawn_local(async move {
            wait_for_bridge().await;
            match login_user(email_val, pass_val).await {
                Ok(user_js) => {
                    if let Ok(user) = serde_wasm_bindgen::from_value::<User>(user_js) {
                        save_user(&user);
                        on_login.call(user);
                        navigate("/", Default::default());
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
