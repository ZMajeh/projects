use leptos::*;
use leptos_router::use_navigate;
use crate::models::User;
use crate::api::login_user;
use crate::utils::{save_user, wait_for_bridge};

#[component]
pub fn Login(on_login: Callback<User>) -> impl IntoView {
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);
    let navigate = use_navigate();

    let on_login_click = move |_| {
        set_loading.set(true);
        set_error.set(None);
        let navigate = navigate.clone();
        spawn_local(async move {
            match login_user().await {
                Ok(user_js) => {
                    if let Ok(user) = serde_wasm_bindgen::from_value::<User>(user_js) {
                        save_user(&user);
                        on_login.call(user);
                        navigate("/", Default::default());
                    }
                }
                Err(e) => { 
                    let error_msg = e.as_string().unwrap_or_else(|| "Google Login failed.".to_string());
                    set_error.set(Some(error_msg)); 
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div style="display: flex; justify-content: center; align-items: center; height: 100vh; flex-direction: column; padding: 1rem; background-color: #f0f2f5;">
            <div class="container" style="max-width: 400px; text-align: center; width: 100%; padding: 2rem; box-shadow: 0 10px 25px rgba(0,0,0,0.1);">
                <div style="margin-bottom: 2rem;">
                    <h1 style="color: var(--primary); font-size: 2.5rem; font-weight: 900; margin: 0; text-transform: uppercase; letter-spacing: 3px;">"Anand"</h1>
                    <h2 style="color: #34495e; font-size: 1.2rem; font-weight: 700; margin: 0; opacity: 0.8;">"Lodge Management"</h2>
                </div>
                
                <p style="margin-bottom: 2rem; color: #666;">"Please sign in with your Google account to access the dashboard."</p>

                {move || error.get().map(|err| view! { <p style="color: #e74c3c; font-size: 0.9rem; margin-bottom: 1rem; font-weight: bold;">{err}</p> })}
                
                <button 
                    on:click=on_login_click 
                    style="width: 100%; background-color: #4285F4; color: white; border: none; padding: 12px; border-radius: 4px; font-weight: bold; display: flex; align-items: center; justify-content: center; gap: 10px; transition: background 0.3s;"
                    disabled=loading
                >
                    {move || if loading.get() { 
                        view! { <span>"Signing in..."</span> }.into_view() 
                    } else { 
                        view! { 
                            <>
                                <svg width="18" height="18" viewBox="0 0 18 18" xmlns="http://www.w3.org/2000/svg">
                                    <path d="M17.64 9.2c0-.637-.057-1.251-.164-1.84H9v3.481h4.844c-.209 1.125-.843 2.078-1.796 2.717v2.258h2.908c1.702-1.567 2.684-3.874 2.684-6.615z" fill="#4285F4"/>
                                    <path d="M9 18c2.43 0 4.467-.806 5.956-2.184l-2.908-2.259c-.806.54-1.837.86-3.048.86-2.344 0-4.328-1.584-5.036-3.711H.957v2.332C2.438 15.983 5.482 18 9 18z" fill="#34A853"/>
                                    <path d="M3.964 10.71c-.18-.54-.282-1.117-.282-1.71s.102-1.17.282-1.71V4.958H.957C.347 6.173 0 7.548 0 9s.347 2.827.957 4.042l3.007-2.332z" fill="#FBBC05"/>
                                    <path d="M9 3.58c1.321 0 2.508.454 3.44 1.345l2.582-2.58C13.463.891 11.426 0 9 0 5.482 0 2.438 2.017.957 4.958L3.964 7.29C4.672 5.163 6.656 3.58 9 3.58z" fill="#EA4335"/>
                                </svg>
                                "Sign in with Google"
                            </>
                        }.into_view()
                    }}
                </button>
            </div>
        </div>
    }
}
