use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = addTestLodge)]
    async fn add_test_lodge() -> Result<JsValue, JsValue>;
}

#[component]
fn App() -> impl IntoView {
    let (status, set_status) = create_signal("Ready".to_string());
    let (loading, set_loading) = create_signal(false);

    let on_click = move |_| {
        set_loading.set(true);
        set_status.set("Connecting to Firebase...".to_string());
        
        spawn_local(async move {
            match add_test_lodge().await {
                Ok(id) => {
                    let id_str = id.as_string().unwrap_or_else(|| "unknown".to_string());
                    set_status.set(format!("Success! Created Lodge with ID: {}", id_str));
                }
                Err(e) => {
                    logging::error!("Firebase Error: {:?}", e);
                    set_status.set("Error: Failed to write to Firebase. Check console and Firestore rules.".to_string());
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container">
            <h1>"Welcome to Lodge Management System"</h1>
            <p>"Phase 1: Foundation & Hello World (Rust + WASM + Firebase)"</p>
            
            <button on:click=on_click disabled=loading>
                {move || if loading.get() { "Writing..." } else { "Test Database Connection" }}
            </button>

            <p style="margin-top: 20px; color: #666;">
                "Status: " {move || status.get()}
            </p>

            {move || if status.get().starts_with("Success") {
                view! { <p style="color: green;">"✅ Connection verified! Record added to 'lodges' collection."</p> }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
