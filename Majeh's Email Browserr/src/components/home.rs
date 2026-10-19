use leptos::*;
use crate::components::icons::Icon;
use crate::components::navigation::Tab;
use crate::tokens::OAuthToken;

/// Home view - the app's public homepage. Explains what the app does, why it
/// needs Gmail read-only access, how the client-side architecture protects your
/// data, and how to get started. Fully visible without unlocking anything:
/// the vault is created and unlocked from Settings.
#[component]
pub fn Home(
    connected_accounts: RwSignal<Vec<OAuthToken>>,
    vault_exists: bool,
    set_active_tab: WriteSignal<Tab>,
) -> impl IntoView {
    let features = [
        (
            "mail",
            "A fast three-pane mail client",
            "Inbox on the left, message list in the middle, reading pane on the right - the way a mail client should feel.",
        ),
        (
            "shield",
            "Read-only Google sign-in",
            "Connect Gmail over OAuth with read-only access. The app can never send, delete or modify your mail.",
        ),
        (
            "lock",
            "Your tokens, encrypted locally",
            "OAuth tokens live in an AES-256-GCM vault in your browser, opened only by your vault password. There is no server.",
        ),
        (
            "download",
            "Keep and back up your mail",
            "Fetched mail is cached on your device so it survives restarts, and can be exported or re-imported as a JSON backup.",
        ),
    ];

    let steps = [
        (
            "1",
            "Create your vault",
            "Open Settings and set a vault password under \"Connected Accounts\". It encrypts your tokens in localStorage - it cannot be recovered if you lose it.",
        ),
        (
            "2",
            "Connect Gmail",
            "In Settings, tap \"Connect Gmail\" and sign in with Google. Approve the read-only mail permission.",
        ),
        (
            "3",
            "Read your mail",
            "Your account appears in the sidebar. Click it to fetch your inbox, then use \"Fetch by date\" to pull older mail.",
        ),
    ];

    let account_count = move || connected_accounts.get().len();

    view! {
        <div class="page-scroll">
            <div class="page-card home-view">
                <div class="home-hero">
                    <div class="brand-logo home-logo"></div>
                    <h1>"Majeh's Email Browserr"</h1>
                    <p class="tagline">
                        "A secure, local way to browse your inbox."
                    </p>
                    <p class="home-lede">
                        "Majeh's Email Browserr is an intuitive dashboard designed to let users easily read and browse their Gmail inbox. The application requires read-only access to display your message subjects, senders, and email bodies directly within the interface."
                    </p>
                </div>

                <div class="home-privacy-banner">
                    <h3>
                        <Icon name="shield"/>
                        "100% Private & Serverless Architecture"
                    </h3>
                    <p>
                        "Your security is our priority. Majeh's Email Browserr operates entirely on the client side within your local web browser, hosted as a static web app via Firebase Hosting. All email fetches using the gmail.readonly API are processed locally on your device. We do not own, maintain, or utilize an external database server, and your private email data is never tracked or transmitted to any third party. Your authentication tokens and fetched email data are held only in your own browser - the tokens encrypted in a local vault, the mail in your browser's local storage - and are deleted by you at any time."
                    </p>
                </div>

                <h2 class="home-heading">"What this app does"</h2>
                <div class="home-features">
                    {features
                        .into_iter()
                        .map(|(icon, title, body)| {
                            view! {
                                <div class="home-feature">
                                    <span class="row-icon blue"><Icon name=icon/></span>
                                    <div class="row-text">
                                        <span class="row-title">{title}</span>
                                        <span class="row-sub">{body}</span>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>

                <h2 class="home-heading">"Getting started"</h2>
                <ol class="home-steps">
                    {steps
                        .into_iter()
                        .map(|(num, title, body)| {
                            view! {
                                <li class="home-step">
                                    <span class="home-step-num">{num}</span>
                                    <div class="row-text">
                                        <span class="row-title">{title}</span>
                                        <span class="row-sub">{body}</span>
                                    </div>
                                </li>
                            }
                        })
                        .collect_view()}
                </ol>

                <div class="home-status">
                    {move || {
                        let count = account_count();
                        if count > 0 {
                            view! {
                                <p class="home-status-text">
                                    <span class="home-dot ok"></span>
                                    {format!(
                                        "{count} account(s) connected. Open Settings to manage the vault, or pick an account in the sidebar."
                                    )}
                                </p>
                            }.into_view()
                        } else if vault_exists {
                            view! {
                                <p class="home-status-text">
                                    <span class="home-dot warn"></span>
                                    "A vault exists but is locked. Open Settings and unlock it to load your accounts."
                                </p>
                            }.into_view()
                        } else {
                            view! {
                                <p class="home-status-text">
                                    <span class="home-dot"></span>
                                    "No vault yet - start with step 1 below to create one."
                                </p>
                            }.into_view()
                        }
                    }}
                    <div class="home-cta">
                        <button
                            class="btn-primary"
                            on:click=move |_| set_active_tab.set(Tab::Settings)
                        >
                            <Icon name="settings"/>
                            "Open Settings"
                        </button>
                        <button
                            class="btn-secondary"
                            on:click=move |_| set_active_tab.set(Tab::Mailbox)
                        >
                            <Icon name="mail"/>
                            "Go to Mailbox"
                        </button>
                    </div>
                </div>

                <p class="home-legal">
                    <a href="/privacy.html" target="_blank" rel="noopener noreferrer">
                        "Privacy Policy"
                    </a>
                    <span>" · "</span>
                    <a href="/terms.html" target="_blank" rel="noopener noreferrer">
                        "Terms of Service"
                    </a>
                    <span>" · "</span>
                    <a href="mailto:zmajeh@gmail.com">"Contact"</a>
                </p>
            </div>
        </div>
    }
}
