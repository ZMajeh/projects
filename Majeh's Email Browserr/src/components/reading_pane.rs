use leptos::*;
use crate::components::icons::Icon;
use crate::data::Email;

/// Right pane: full email view with reply actions. Shows a placeholder when
/// nothing is selected (desktop) and a back button on mobile.
#[component]
pub fn ReadingPane(
    emails: RwSignal<Vec<Email>>,
    selected_id: RwSignal<Option<usize>>,
) -> impl IntoView {
    let email = move || selected_id.get().and_then(|id| emails.get().into_iter().find(|e| e.id == id));

    view! {
        <section class="reading-pane" class:open=move || selected_id.get().is_some()>
            {move || match email() {
                Some(e) => view! {
                    <div class="reading-inner">
                        <div class="reading-topbar">
                            <button class="icon-btn mobile-only" on:click=move |_| selected_id.set(None) aria-label="Back">
                                <Icon name="chevron-left"/>
                            </button>
                            <h2 class="reading-subject">{e.subject.clone()}</h2>
                            <div class="reading-actions">
                                <button class="icon-btn" aria-label="Star"><Icon name="star"/></button>
                                <button class="icon-btn" aria-label="Archive"><Icon name="archive"/></button>
                                <button class="icon-btn" aria-label="Delete"><Icon name="trash"/></button>
                            </div>
                        </div>
                        <div class="reading-header">
                            <div class="avatar lg" style=format!("background:{}", e.color)>{e.initial}</div>
                            <div class="reading-from">
                                <span class="sender">{e.sender.clone()} <span class="addr">{format!("<{}>", e.address)}</span></span>
                                <span class="to">"To me"</span>
                            </div>
                            <span class="time">{e.time.clone()}</span>
                        </div>
                        <div class="reading-body">
                            {e.body.iter().map(|p| view! { <p>{*p}</p> }).collect_view()}
                        </div>
                        <div class="reading-footer">
                            <button class="btn-action"><Icon name="reply"/>"Reply"</button>
                            <button class="btn-action"><Icon name="reply-all"/>"Reply All"</button>
                            <button class="btn-action"><Icon name="forward"/>"Forward"</button>
                        </div>
                    </div>
                },
                None => view! {
                    <div class="reading-placeholder">
                        <Icon name="mail" class="placeholder-icon"/>
                        <p>"Select an email to read"</p>
                    </div>
                },
            }}
        </section>
    }
}
