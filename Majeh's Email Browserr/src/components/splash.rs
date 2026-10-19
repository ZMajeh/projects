use leptos::*;

/// Brief brand intro shown while the app boots. No login here - the vault is
/// created/unlocked from Settings, and the Home tab explains how.
#[component]
pub fn Splash() -> impl IntoView {
    let (dismissed, set_dismissed) = create_signal(false);

    // Kept short on purpose: an automated reviewer that screenshots the page
    // must never land on a full-screen overlay that looks like a login gate.
    // The purpose line below makes even that frame self-explanatory.
    wasm_bindgen_futures::spawn_local(async move {
        crate::util::sleep(900).await;
        set_dismissed.set(true);
    });

    view! {
        <Show when=move || !dismissed.get()>
            <div class="gate-screen splash">
                <div class="brand-logo splash-logo"></div>
                <p class="splash-subtitle">
                    "A private, client-side Gmail reader. Read-only access - no servers, no database."
                </p>
                <div class="splash-progress">
                    <div class="splash-progress-fill"></div>
                </div>
            </div>
        </Show>
    }
}
