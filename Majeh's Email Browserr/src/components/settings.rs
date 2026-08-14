use leptos::*;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <div class="settings-view">
            <h2>"Settings"</h2>
            <div class="setting-item">
                <label>"Add Email Address"</label>
                <input type="text" placeholder="user@example.com" />
                <button>"Add"</button>
            </div>
        </div>
    }
}
