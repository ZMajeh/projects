use leptos::*;

mod app;
mod components;
mod crypto;
mod data;
mod oauth;
mod tokens;
mod util;
use app::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });

    // The static homepage block in index.html exists so crawlers and
    // pre-WASM visitors see real content. Now that the app is mounted, hand
    // the page over to it.
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id("static-home") {
            el.remove();
        }
    }
}
