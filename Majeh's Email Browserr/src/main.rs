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

    // The OAuth popup lands on `/auth/callback/google?code=...`. It only has
    // to hand the authorization code back to the window that opened it and
    // close itself - the token exchange happens in the opener, where the PKCE
    // verifier and the unlocked vault live. Booting the full app here would
    // be wasted work and would flash the whole UI in a small window.
    if crate::oauth::is_popup_callback() {
        match crate::oauth::relay_code_to_opener() {
            Ok(()) => crate::util::log_info("popup relayed the authorization code back"),
            Err(e) => crate::util::log_error(&format!("popup could not relay the code: {e}")),
        }
        if let Some(window) = web_sys::window() {
            let _ = window.close();
        }
        return;
    }

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
