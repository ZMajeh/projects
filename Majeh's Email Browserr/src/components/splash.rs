use leptos::*;
use crate::tokens::{create_vault, unlock_vault, vault_exists, OAuthToken};

/// Full-screen splash that acts as a hard gate: it stays until the user
/// either unlocks an existing vault or creates a new vault password.
/// Only then does the app appear, with `connected_accounts` already populated.
#[component]
pub fn Splash(connected_accounts: RwSignal<Vec<OAuthToken>>) -> impl IntoView {
    let (unlocking, set_unlocking) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (dismissed, set_dismissed) = create_signal(false);
    let password = create_rw_signal(String::new());
    let confirm = create_rw_signal(String::new());

    let has_vault = vault_exists();

    let handle_submit = move |_| {
        set_unlocking.set(true);
        set_error.set(None);
        let pw = password.get();
        let tokens = connected_accounts.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = if has_vault {
                // Existing vault → unlock it, populate accounts.
                unlock_vault(&pw).map(|decrypted| {
                    tokens.set(decrypted);
                })
            } else {
                // No vault → create one with this password (empty token list).
                let confirm_pw = confirm.get();
                if pw != confirm_pw {
                    Err("Passwords do not match".to_string())
                } else if pw.is_empty() {
                    Err("Password cannot be empty".to_string())
                } else {
                    create_vault(&pw)
                }
            };

            match result {
                Ok(()) => {
                    set_dismissed.set(true);
                    set_unlocking.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_unlocking.set(false);
                }
            }
        });
    };

    view! {
        <Show when=move || !dismissed.get()>
            <div class="gate-screen splash">
                <div class="brand-logo xl"></div>
                <h1 class="splash-title">"Majeh's Email Browserr"</h1>

                {move || if has_vault {
                    view! {
                        <div class="splash-unlock">
                            <p class="splash-subtitle">"Unlock your vault to continue"</p>
                            <input
                                type="password"
                                placeholder="Vault password"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                                class="splash-password-input"
                            />
                            <button
                                class="btn-unlock"
                                disabled=move || unlocking.get()
                                on:click=handle_submit
                            >
                                {move || if unlocking.get() { "Unlocking…" } else { "Unlock" }}
                            </button>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="splash-unlock">
                            <p class="splash-subtitle">"Create your vault password"</p>
                            <p class="splash-subtitle splash-hint">
                                "This encrypts your OAuth tokens in localStorage."
                            </p>
                            <input
                                type="password"
                                placeholder="New password"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                                class="splash-password-input"
                            />
                            <input
                                type="password"
                                placeholder="Confirm password"
                                prop:value=move || confirm.get()
                                on:input=move |ev| confirm.set(event_target_value(&ev))
                                class="splash-password-input"
                            />
                            <button
                                class="btn-unlock"
                                disabled=move || unlocking.get()
                                on:click=handle_submit
                            >
                                {move || if unlocking.get() { "Creating…" } else { "Create Vault" }}
                            </button>
                        </div>
                    }.into_view()
                }}

                {move || error.get().map(|e| view! {
                    <div class="splash-error">{e}</div>
                })}

                <div class="splash-progress">
                    <div class="splash-progress-fill"></div>
                </div>
            </div>
        </Show>
    }
}