use leptos::*;
use leptos_router::*;
use crate::models::User;
use crate::utils::{clear_user, wait_for_bridge};
use crate::api::{sign_out_user, authorize_google_drive, is_drive_authorized};

#[component]
pub fn DashboardLayout(user: User, on_logout: Callback<()>, children: Children) -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);
    let (drive_auth, set_drive_auth) = create_signal(is_drive_authorized());

    let handle_logout = move |_| {
        clear_user();
        spawn_local(async move {
            wait_for_bridge().await;
            let _ = sign_out_user().await;
            on_logout.call(());
        });
    };

    let handle_drive_auth = move |_| {
        spawn_local(async move {
            wait_for_bridge().await;
            if let Ok(_) = authorize_google_drive().await {
                set_drive_auth.set(is_drive_authorized());
            }
        });
    };

    view! { 
        <div class="app-layout">
            <div class=move || format!("sidebar-overlay no-print {}", if menu_open.get() { "show" } else { "" }) on:click=move |_| set_menu_open.set(false)></div>
            <nav class=move || format!("sidebar no-print {}", if menu_open.get() { "open" } else { "" })>
                <div style="padding: 1.5rem; border-bottom: 2px solid var(--primary); margin-bottom: 1rem; background: #f8f9fa;">
                    <h1 style="color: var(--primary); font-size: 1.8rem; font-weight: 900; margin: 0; text-transform: uppercase; letter-spacing: 2px; text-shadow: 1px 1px 2px rgba(0,0,0,0.1);">"Anand"</h1>
                    <h2 style="color: #34495e; font-size: 1.1rem; font-weight: 700; margin: 0; opacity: 0.8;">"Lodge Manager"</h2>
                </div>
                <A href="" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active" exact=true>"Overview"</A>
                <A href="rooms" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Rooms"</A>
                <A href="customers" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Customers"</A>
                <A href="bookings" on:click=move |_| set_menu_open.set(false) class="nav-link" active_class="active">"Bookings"</A>
                
                <div style="margin-top: auto; padding: 1rem; border-top: 1px solid #ddd; font-size: 0.85rem;">
                    {move || if !drive_auth.get() {
                        view! {
                            <div style="padding: 0 1rem 1rem 1rem;">
                                <p style="color: #e67e22; font-size: 0.7rem; margin-bottom: 5px;">"⚠ Drive Not Authorized"</p>
                                <button on:click=handle_drive_auth style="background-color: #4285F4; width: 100%; border-radius: 6px; font-size: 0.75rem; padding: 0.5rem;">"Sign in with Google"</button>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div style="padding: 0 1rem 1rem 1rem;">
                                <p style="color: #27ae60; font-size: 0.7rem; margin-bottom: 5px;">"✅ Drive Connected"</p>
                            </div>
                        }.into_view()
                    }}
                    <p style="color: #7f8c8d; overflow: hidden; text-overflow: ellipsis; margin-bottom: 8px; padding-left: 1rem;">{user.email}</p>
                    <button on:click=handle_logout style="background-color: #e74c3c; width: calc(100% - 2rem); margin: 0 1rem; border-radius: 6px;">"Logout"</button>
                </div>
            </nav>
            <main class="content">
                <header class="mobile-header no-print">
                    <button on:click=move |_| set_menu_open.update(|v| *v = !*v) style="background: none; color: black; font-size: 1.5rem; padding: 0;">"☰"</button>
                    <strong style="color: var(--primary); font-weight: 900; font-size: 1.2rem; letter-spacing: 1px;">"ANAND LODGE"</strong>
                    <div style="width: 30px;"></div>
                </header>
                <div style="padding: 1rem;">{children()}</div>
            </main>
        </div> 
    }
}
