use leptos::*;

#[component]
pub fn About() -> impl IntoView {
    view! {
        <div class="about-view">
            <h2>"About Browserr"</h2>
            <p>"A high-performance email browser."</p>
            <h3>"Tech Stack"</h3>
            <ul>
                <li>"Rust"</li>
                <li>"Leptos"</li>
                <li>"WebAssembly"</li>
            </ul>
        </div>
    }
}
