use leptos::*;

/// Brief brand intro shown while the app boots. No login here - the vault is
/// created/unlocked from Settings, and the Home tab explains how.
#[component]
pub fn Splash() -> impl IntoView {
    let (dismissed, set_dismissed) = create_signal(false);

    wasm_bindgen_futures::spawn_local(async move {
        crate::util::sleep(1500).await;
        set_dismissed.set(true);
    });

    view! {
        <Show when=move || !dismissed.get()>
            <div class="gate-screen splash">
                <div class="brand-logo splash-logo"></div>
                <div class="splash-progress">
                    <div class="splash-progress-fill"></div>
                </div>
            </div>
        </Show>
    }
}
