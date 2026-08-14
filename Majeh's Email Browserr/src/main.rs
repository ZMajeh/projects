use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div class="sidebar">
            <h3>"Browserr"</h3>
            <div class="nav-item">"Inbox"</div>
            <div class="nav-item">"Sent"</div>
            <div class="nav-item">"Drafts"</div>
            <div class="nav-item">"Trash"</div>
        </div>
        <div class="thread-list">
            <div class="email-item">
                <h4>"Welcome to Browserr"</h4>
                <p>"Start browsing your emails with speed..."</p>
            </div>
            <div class="email-item">
                <h4>"Rust + Leptos is Fast"</h4>
                <p>"The power of WASM in your browser."</p>
            </div>
        </div>
        <div class="viewport">
            <h1>"Hello World"</h1>
            <p>"Select an email to read its contents here."</p>
            <hr />
            <div style="color: #888; font-style: italic;">
                "This is a placeholder for the email detail view."
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
