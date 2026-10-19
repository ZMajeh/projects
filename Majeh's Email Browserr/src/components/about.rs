use leptos::*;
use crate::components::icons::Icon;
use crate::util::site_url;

/// About view: hero with logo + link rows.
#[component]
pub fn About() -> impl IntoView {
    // (icon, title, href) — rows with a href open the page in a new tab.
    let rows: [(&str, &str, Option<String>); 6] = [
        ("book", "Features", None),
        ("globe", "Open Source", None),
        ("bug", "Report a Bug", None),
        ("mail", "Contact", None),
        ("shield", "Privacy Policy", Some(site_url("/privacy.html"))),
        ("info", "Terms of Service", Some(site_url("/terms.html"))),
    ];

    view! {
        <div class="page-scroll">
            <div class="page-card about-view">
                <div class="about-hero">
                    <div class="brand-logo xl"></div>
                    <h1>"Majeh's Email Browserr"</h1>
                    <p class="tagline">"A modern, fast, and secure email browser for your productivity."</p>
                    <p class="version">"Version 1.0.0"</p>
                </div>
                <ul class="settings-list">
                    {rows.into_iter().map(|(icon, title, href)| {
                        let row_title = view! {
                            <>
                                <span class="row-icon slate"><Icon name=icon/></span>
                                <div class="row-text"><span class="row-title">{title}</span></div>
                            </>
                        };
                        match href {
                            Some(href) => view! {
                                <li class="settings-row row-link-row">
                                    <a class="row-link" href=href target="_blank" rel="noopener noreferrer">
                                        {row_title}
                                        <Icon name="chevron-right" class="chevron"/>
                                    </a>
                                </li>
                            },
                            None => view! {
                                <li class="settings-row">
                                    {row_title}
                                    <Icon name="chevron-right" class="chevron"/>
                                </li>
                            },
                        }
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}
