use leptos::*;

#[component]
pub fn Mailbox() -> impl IntoView {
    view! {
        <div class="mailbox-view">
            <h2>"Inbox"</h2>
            <div class="email-list">
                <div class="email-item">
                    <div class="sender">"Gemini CLI"</div>
                    <div class="subject">"Welcome to Browserr"</div>
                </div>
            </div>
        </div>
    }
}
