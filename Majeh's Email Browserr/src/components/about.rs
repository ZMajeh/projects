use leptos::*;
use crate::components::icons::Icon;

/// About view: hero with logo + link rows.
#[component]
pub fn About() -> impl IntoView {
    let rows = [
        ("book", "Features"),
        ("globe", "Open Source"),
        ("bug", "Report a Bug"),
        ("mail", "Contact"),
        ("shield", "Privacy Policy"),
        ("info", "Terms of Service"),
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
                    {rows.into_iter().map(|(icon, title)| view! {
                        <li class="settings-row">
                            <span class="row-icon slate"><Icon name=icon/></span>
                            <div class="row-text"><span class="row-title">{title}</span></div>
                            <Icon name="chevron-right" class="chevron"/>
                        </li>
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}
