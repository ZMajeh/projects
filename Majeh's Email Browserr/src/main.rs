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
}
