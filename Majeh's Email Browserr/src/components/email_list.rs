use leptos::*;
use crate::components::icons::Icon;
use crate::data::{Email, Folder};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ListFilter {
    All,
    Unread,
    Starred,
}

/// Middle pane: folder header with filter tabs + scrollable email rows.
#[component]
pub fn EmailList(
    emails: RwSignal<Vec<Email>>,
    active_folder: RwSignal<Folder>,
    filter: RwSignal<ListFilter>,
    query: RwSignal<String>,
    selected_id: RwSignal<Option<usize>>,
    on_select: impl Fn(usize) + Copy + 'static,
    /// True when the active account is a connected OAuth account (refresh applies).
    #[prop(optional)]
    show_refresh: Option<Callback<(), bool>>,
    /// True while the active account's inbox fetch is running (spinner state).
    #[prop(optional)]
    refreshing: Option<Callback<(), bool>>,
    /// Manual inbox refresh (always hits the network, merges by msg_id).
    #[prop(optional)]
    on_refresh: Option<Callback<()>>,
) -> impl IntoView {
    let tabs = [(ListFilter::All, "All"), (ListFilter::Unread, "Unread"), (ListFilter::Starred, "Starred")];

    // Emails matching the active folder + search query (before the All/Unread/Starred tab).
    let scoped = move || {
        let folder = active_folder.get();
        let q = query.get().to_lowercase();
        emails.get().into_iter().filter(|e| {
            let folder_ok = match folder {
                Folder::Starred => e.starred,
                f => e.folder == f,
            };
            let query_ok = q.is_empty()
                || e.sender.to_lowercase().contains(&q)
                || e.subject.to_lowercase().contains(&q)
                || e.preview.to_lowercase().contains(&q)
                || e.address.to_lowercase().contains(&q);
            folder_ok && query_ok
        }).collect::<Vec<_>>()
    };

    view! {
        <section class="email-list-pane">
            <div class="list-header">
                <h2>{move || active_folder.get().label()}</h2>
                <span class="unread-count">
                    {move || scoped().iter().filter(|e| e.unread).count()} " unread"
                </span>
                {move || show_refresh.as_ref().map(|show| {
                    let show = show.clone();
                    let refreshing = refreshing.clone();
                    let on_refresh = on_refresh.clone();
                    view! {
                        <Show when=move || show.call(())>
                            <button
                                class="btn-inbox-refresh"
                                title="Refresh inbox"
                                aria-label="Refresh inbox"
                                disabled=move || refreshing.as_ref().map(|r| r.call(())).unwrap_or(false)
                                on:click=move |_| { if let Some(refresh) = on_refresh.as_ref() { refresh.call(()); } }
                            >
                                <Icon
                                    name="refresh"
                                    class=if refreshing.as_ref().map(|r| r.call(())).unwrap_or(false) { "spin" } else { "" }
                                />
                                <span>
                                    {move || if refreshing.as_ref().map(|r| r.call(())).unwrap_or(false) { "Refreshing…" } else { "Refresh" }}
                                </span>
                            </button>
                        </Show>
                    }
                })}
                <button class="btn-filter"><Icon name="sliders"/>"Filter"</button>
            </div>
            <div class="filter-tabs">
                {tabs.into_iter().map(|(f, label)| view! {
                    <button
                        class="tab"
                        class:active=move || filter.get() == f
                        on:click=move |_| filter.set(f)
                    >
                        {label}
                    </button>
                }).collect_view()}
            </div>
            <Show
                when=move || {
                    let f = filter.get();
                    scoped().iter().any(|e| match f {
                        ListFilter::All => true,
                        ListFilter::Unread => e.unread,
                        ListFilter::Starred => e.starred,
                    })
                }
                fallback=move || view! {
                    <div class="list-empty">
                        <Icon name="mail" class="placeholder-icon"/>
                        <p>"Nothing here yet"</p>
                    </div>
                }
            >
                <ul class="email-list">
                    <For
                        each=move || {
                            let f = filter.get();
                            scoped().into_iter().filter(|e| match f {
                                ListFilter::All => true,
                                ListFilter::Unread => e.unread,
                                ListFilter::Starred => e.starred,
                            }).collect::<Vec<_>>()
                        }
                        key=|e| (e.id, e.starred, e.unread)
                        children=move |e: Email| {
                        let id = e.id;
                        view! {
                            <li
                                class="email-item"
                                class:unread=move || e.unread
                                class:selected=move || selected_id.get() == Some(id)
                                on:click=move |_| on_select(id)
                            >
                                <span class="unread-dot" class:visible=move || e.unread></span>
                                <div class="avatar" style=format!("background:{}", e.color)>{e.initial}</div>
                                <div class="email-meta">
                                    <div class="row1">
                                        <span class="sender">{e.sender}</span>
                                        <span class="time">{e.time}</span>
                                    </div>
                                    <div class="subject">{e.subject}</div>
                                    <div class="preview">{e.preview}</div>
                                </div>
                                <button
                                    class="star-btn"
                                    class:starred=move || e.starred
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        emails.update(|list| {
                                            if let Some(x) = list.iter_mut().find(|x| x.id == id) {
                                                x.starred = !x.starred;
                                            }
                                        });
                                    }
                                    aria-label="Star"
                                >
                                    <Icon name="star"/>
                                </button>
                            </li>
                        }
                    }
                />
                </ul>
            </Show>
        </section>
    }
}
