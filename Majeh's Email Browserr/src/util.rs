//! Small shared helpers.

/// The live origin of the deployed app.
///
/// Used for every absolute public link (privacy policy, terms of service) so
/// that the homepage link, the in-app links and the OAuth consent screen all
/// point at the exact same URL.
///
/// IMPORTANT: when the domain changes, update this constant AND the "Privacy
/// policy" URL in the Google Cloud OAuth consent screen — Google requires the
/// homepage link to match the consent screen link.
pub const SITE_URL: &str = "https://majehsemailviewer.web.app";

/// Absolute URL for a path on the site (e.g. `site_url("/privacy.html")`).
pub fn site_url(path: &str) -> String {
    format!("{}{}", SITE_URL, path)
}

/// Await a JS timer without pulling in an extra crate.
pub async fn sleep(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// Console logging used to trace the OAuth / vault flow.
///
/// Every step writes a `[majeh] ...` line to the browser console so a failed
/// sign-in can be diagnosed without guesswork. `console.error` is used for
/// failures so they stand out in the console filter.
pub fn log_info(msg: &str) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("[majeh] {msg}")));
}

/// Log a failure (shown in red in the browser console).
pub fn log_error(msg: &str) {
    web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!("[majeh] {msg}")));
}

/// Log a labelled value, e.g. `log_kv("state", &state)`.
#[allow(dead_code)]
pub fn log_kv(label: &str, value: &str) {
    log_info(&format!("{label} = {value}"));
}
