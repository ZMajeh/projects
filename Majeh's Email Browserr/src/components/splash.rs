use leptos::*;

/// Full-screen splash shown for a few seconds on launch: centered brand logo
/// with an indefinite (sliding) progress bar. Purely visual — everything is
/// client-side, so no login gate is needed.
#[component]
pub fn Splash() -> impl IntoView {
    view! {
        <div class="gate-screen splash">
            <div class="brand-logo xl"></div>
            <h1 class="splash-title">"Majeh's Email Browserr"</h1>
            <p class="splash-subtitle">"A more organized you, a brighter tomorrow."</p>
            <div class="splash-progress">
                <div class="splash-progress-fill"></div>
            </div>
        </div>
    }
}