use leptos::*;
use wasm_bindgen::JsCast;
use crate::components::icons::Icon;
use crate::data::{FetchTask, TaskStatus};
use crate::tokens::OAuthToken;

/// Top strip of the Mailbox view: per-task fetch progress bars
/// ("Fetching 20 emails for x@gmail.com — 12/20"), fetch errors, and a
/// collapsible manual date-range fetch control (one task per account+range).
#[component]
pub fn FetchPanel(
    connected_accounts: RwSignal<Vec<OAuthToken>>,
    tasks: RwSignal<Vec<FetchTask>>,
    error: RwSignal<Option<String>>,
    on_fetch_range: impl Fn(String, String, String) + Copy + 'static,
) -> impl IntoView {
    let (expanded, set_expanded) = create_signal(false);

    let read_input = |id: &str| -> String {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|el| el.value())
            .unwrap_or_default()
    };
    let read_select = |id: &str| -> String {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|el| el.dyn_into::<web_sys::HtmlSelectElement>().ok())
            .map(|el| el.value())
            .unwrap_or_default()
    };

    view! {
        <div class="fetch-panel">
            <div class="fetch-panel-head">
                <button
                    class="fetch-toggle"
                    class:open=move || expanded.get()
                    on:click=move |_| set_expanded.update(|e| *e = !*e)
                >
                    <Icon name="clock"/>
                    <span>"Fetch by date"</span>
                    <Icon name="chevron-down" class="chevron"/>
                </button>
            </div>
            <Show when=move || expanded.get()>
                <div class="fetch-controls">
                    <label class="fetch-field">
                        "From"
                        <input type="date" id="fetch-from"/>
                    </label>
                    <label class="fetch-field">
                        "To"
                        <input type="date" id="fetch-to"/>
                    </label>
                    <select id="fetch-account" aria-label="Account">
                        <option value="">"All connected accounts"</option>
                        {move || {
                            let accounts = connected_accounts.get();
                            accounts
                                .into_iter()
                                .map(|t| {
                                    let email = t.email.clone();
                                    view! { <option value=email.clone()>{email}</option> }
                                })
                                .collect_view()
                        }}
                    </select>
                    <button
                        class="btn-fetch"
                        on:click=move |_| {
                            let from = read_input("fetch-from");
                            let to = read_input("fetch-to");
                            let account = read_select("fetch-account");
                            on_fetch_range(from, to, account);
                        }
                    >
                        "Fetch Range"
                    </button>
                </div>
            </Show>
            {move || {
                error
                    .get()
                    .map(|e| view! {
                        <div class="fetch-error">
                            <span>{e}</span>
                            <button
                                class="icon-btn sm"
                                aria-label="Dismiss"
                                on:click=move |_| error.set(None)
                            >
                                "✕"
                            </button>
                        </div>
                    })
            }}
            {move || {
                let list = tasks.get();
                if list.is_empty() {
                    return None;
                }
                Some(
                    view! {
                        <div class="fetch-tasks">
                            {list
                                .iter()
                                .map(|t| {
                                    let text = t.progress_text();
                                    let frac = t.fraction();
                                    let is_err = matches!(t.status, TaskStatus::Error(_));
                                    let is_done = t.status == TaskStatus::Done;
                                    let indeterminate = !is_done && !is_err && frac.is_none();
                                    let width = if is_done {
                                        100.0
                                    } else {
                                        frac.map(|f| f * 100.0).unwrap_or(0.0)
                                    };
                                    view! {
                                        <div
                                            class="fetch-task"
                                            class:error=is_err
                                            class:done=is_done
                                        >
                                            <div class="task-text">{text}</div>
                                            <div
                                                class="progress-track"
                                                class:indeterminate=indeterminate
                                            >
                                                <div
                                                    class="progress-fill"
                                                    style:width=if indeterminate {
                                                        String::new()
                                                    } else {
                                                        format!("{width:.1}%")
                                                    }
                                                ></div>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    },
                )
            }}
        </div>
    }
}