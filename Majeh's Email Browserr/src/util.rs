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
