use leptos::*;
use crate::models::{Booking, Customer};

#[component]
pub fn BookingDetails(
    booking: Booking, 
    customers: Vec<Customer>, 
    on_close: Callback<()>
) -> impl IntoView {
    let today = js_sys::Date::new_0().to_iso_string().as_string().unwrap()[..10].to_string();
    
    view! {
        <div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: #555; z-index: 10000; overflow-y: auto; color: #000; padding: 20px 0;" class="bill-overlay">
            <div style="max-width: 850px; margin: 0 auto 15px auto; display: flex; justify-content: space-between; padding: 0 20px;" class="no-print">
                <button on:click=move |_| on_close.call(()) style="background: #e74c3c; padding: 10px 20px; border-radius: 4px; border: none; color: white; cursor: pointer; font-weight: bold;">"← Close"</button>
                <button on:click=move |_| { let _ = window().print(); } style="background: #2ecc71; padding: 10px 30px; font-weight: bold; border-radius: 4px; border: none; color: white; cursor: pointer; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">"PRINT DETAILS"</button>
            </div>

            <div style="width: 21cm; min-height: 29.7cm; margin: 0 auto; background: white; padding: 1cm 1.5cm; box-sizing: border-box; position: relative; box-shadow: 0 0 20px rgba(0,0,0,0.3); border-radius: 2px;" class="printable-area">
                <div style="position: absolute; top: 500px; left: 50%; transform: translate(-50%, -50%) rotate(-45deg); font-size: 8rem; color: rgba(0,0,0,0.01); pointer-events: none; font-weight: 900; white-space: nowrap; z-index: 0;" class="watermark">"ANAND LODGE"</div>

                <div style="position: relative; z-index: 1;">
                    // Header (Consistent with Bill)
                    <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 25px; border-bottom: 4px double #2c3e50; padding-bottom: 15px;">
                        <div style="flex: 1;">
                            <h1 style="margin: 0; font-size: 2.2rem; color: #2c3e50; text-transform: uppercase; letter-spacing: 2px; font-weight: 900;">"ANAND LODGE"</h1>
                            <p style="margin: 5px 0 0 0; font-size: 0.8rem; color: #34495e; line-height: 1.4;">
                                "Front of bus-stand, Gangakhed" | "Ph: +91 70660 58468"<br/>
                                "Email: vijaymundhe90@gmail.com"
                            </p>
                        </div>
                        <div style="text-align: right;">
                            <div style="background: #2c3e50; color: white; padding: 8px 20px; border-radius: 4px; margin-bottom: 10px;">
                                <h2 style="margin: 0; font-size: 0.9rem; letter-spacing: 1px; text-transform: uppercase;">"BOOKING RECORD"</h2>
                            </div>
                            <p style="margin: 0; font-size: 0.8rem; font-weight: 700;">"Date: " {today}</p>
                            <p style="margin: 0; font-size: 0.8rem;">"Booking ID: AL-" {booking.id.clone().unwrap_or_default().chars().take(8).collect::<String>().to_uppercase()}</p>
                        </div>
                    </div>

                    // Stay Summary
                    <div style="background: #f8f9fa; border: 1px solid #eee; border-radius: 8px; padding: 15px; margin-bottom: 25px; display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px;">
                        <div><label style="display: block; font-size: 0.65rem; text-transform: uppercase; color: #7f8c8d; font-weight: 700;">"Room Number"</label><span style="font-weight: 700; color: #2c3e50;">"Room " {booking.room_number.clone()}</span></div>
                        <div><label style="display: block; font-size: 0.65rem; text-transform: uppercase; color: #7f8c8d; font-weight: 700;">"Arrival Date"</label><span style="font-weight: 700; color: #2c3e50;">{booking.check_in_date.clone()}</span></div>
                        <div><label style="display: block; font-size: 0.65rem; text-transform: uppercase; color: #7f8c8d; font-weight: 700;">"Departure Date"</label><span style="font-weight: 700; color: #2c3e50;">{booking.check_out_date.clone()}</span></div>
                        <div><label style="display: block; font-size: 0.65rem; text-transform: uppercase; color: #7f8c8d; font-weight: 700;">"Current Status"</label><span style="font-weight: 700; color: #27ae60;">{booking.status.clone().to_uppercase()}</span></div>
                    </div>

                    // Detailed Guest Information
                    <h3 style="border-left: 5px solid #2c3e50; padding-left: 10px; font-size: 1.1rem; color: #2c3e50; margin-bottom: 20px; text-transform: uppercase; letter-spacing: 1px;">"Guest Details & Identification"</h3>
                    
                    <div style="display: flex; flex-direction: column; gap: 30px;">
                        <For each=move || customers.clone() key=|c| c.id.clone().unwrap_or_default() children=move |c| {
                            view! {
                                <div style="border: 1px solid #ddd; border-radius: 8px; overflow: hidden; page-break-inside: avoid; break-inside: avoid; margin-bottom: 10px;">
                                    <div style="background: #2c3e50; color: white; padding: 8px 15px; display: flex; justify-content: space-between; align-items: center;">
                                        <span style="font-weight: 700; letter-spacing: 1px;">{c.full_name.clone()}</span>
                                        <span style="font-size: 0.7rem; background: rgba(255,255,255,0.2); padding: 2px 8px; border-radius: 4px;">{if c.id.as_deref() == Some(&booking.customer_id) { "PRIMARY GUEST" } else { "CO-GUEST" }}</span>
                                    </div>
                                    <div style="padding: 15px; display: flex; gap: 20px;">
                                        // Text Info
                                        <div style="flex: 1;">
                                            <table style="width: 100%; font-size: 0.85rem; border-spacing: 0 8px;">
                                                <tr><td style="color: #7f8c8d; width: 100px;">"Phone"</td><td style="font-weight: 600;">{c.phone.clone()}</td></tr>
                                                <tr><td style="color: #7f8c8d;">"Aadhaar No"</td><td style="font-weight: 600; letter-spacing: 1px;">{c.aadhaar.clone()}</td></tr>
                                                <tr><td style="color: #7f8c8d;">"Age / Gender"</td><td style="font-weight: 600;">{format!("{} / {}", c.age.clone().unwrap_or_else(|| "--".to_string()), c.gender.clone().unwrap_or_else(|| "--".to_string()))}</td></tr>
                                                <tr><td style="color: #7f8c8d;">"Verification"</td><td style="font-weight: 600; color: #27ae60;">{if c.verified { "✓ VERIFIED" } else { "⚠ UNVERIFIED" }}</td></tr>
                                            </table>
                                        </div>
                                        
                                        // Guest Photo
                                        <div style="width: 120px; text-align: center;">
                                            <div style="width: 120px; height: 140px; border: 2px solid #eee; border-radius: 4px; overflow: hidden; background: #f9f9f9; display: flex; align-items: center; justify-content: center;">
                                                {if let Some(img) = c.photo_url.clone() {
                                                    if img.contains("drive.google.com") {
                                                        let id = img.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).unwrap_or("").to_string();
                                                        let url_res = create_resource(move || id.clone(), |fid| async move {
                                                            crate::api::get_drive_thumbnail(fid).await.ok().and_then(|v| v.as_string())
                                                        });
                                                        view! { 
                                                            <Suspense fallback=move || view! { <p style="font-size: 0.6rem;">"Loading..."</p> }>
                                                                {move || url_res.get().flatten().map(|u| view! { <img src=u style="width: 100%; height: 100%; object-fit: cover;" /> })}
                                                            </Suspense>
                                                        }.into_view()
                                                    } else {
                                                        view! { <img src=img style="width: 100%; height: 100%; object-fit: cover;" /> }.into_view()
                                                    }
                                                } else {
                                                    view! { <span style="font-size: 0.6rem; color: #ccc;">"NO PHOTO"</span> }.into_view()
                                                }}
                                            </div>
                                            <p style="margin: 5px 0 0 0; font-size: 0.6rem; font-weight: 700; color: #7f8c8d; text-transform: uppercase;">"Guest Photo"</p>
                                        </div>
                                        </div>

                                        // ID Documents
                                        <div style="background: #fcfcfc; border-top: 1px solid #eee; padding: 15px;">
                                        <p style="margin: 0 0 10px 0; font-size: 0.7rem; font-weight: 700; color: #7f8c8d; text-transform: uppercase; letter-spacing: 1px;">"Identity Documents (Aadhaar Card)"</p>
                                        <div style="display: flex; gap: 15px;">
                                            <div style="flex: 1; text-align: center;">
                                                <div style="height: 180px; border: 1px solid #eee; border-radius: 4px; overflow: hidden; background: #fff; display: flex; align-items: center; justify-content: center;">
                                                    {if let Some(img) = c.id_card_url.clone() {
                                                        if img.contains("drive.google.com") {
                                                            let id = img.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).unwrap_or("").to_string();
                                                            let url_res = create_resource(move || id.clone(), |fid| async move {
                                                                crate::api::get_drive_thumbnail(fid).await.ok().and_then(|v| v.as_string())
                                                            });
                                                            view! { 
                                                                <Suspense fallback=move || view! { <p style="font-size: 0.6rem;">"Loading..."</p> }>
                                                                    {move || url_res.get().flatten().map(|u| view! { <img src=u style="max-width: 100%; max-height: 100%; object-fit: contain;" /> })}
                                                                </Suspense>
                                                            }.into_view()
                                                        } else {
                                                            view! { <img src=img style="max-width: 100%; max-height: 100%; object-fit: contain;" /> }.into_view()
                                                        }
                                                    } else {
                                                        view! { <span style="font-size: 0.6rem; color: #ccc;">"ID FRONT NOT SCANNED"</span> }.into_view()
                                                    }}
                                                </div>
                                                <p style="margin: 5px 0 0 0; font-size: 0.6rem; color: #95a5a6;">"Front Side"</p>
                                            </div>
                                            <div style="flex: 1; text-align: center;">
                                                <div style="height: 180px; border: 1px solid #eee; border-radius: 4px; overflow: hidden; background: #fff; display: flex; align-items: center; justify-content: center;">
                                                    {if let Some(img) = c.id_card_back_url.clone() {
                                                        if img.contains("drive.google.com") {
                                                            let id = img.split("/d/").nth(1).and_then(|s| s.split('/').next()).and_then(|s| s.split('?').next()).unwrap_or("").to_string();
                                                            let url_res = create_resource(move || id.clone(), |fid| async move {
                                                                crate::api::get_drive_thumbnail(fid).await.ok().and_then(|v| v.as_string())
                                                            });
                                                            view! { 
                                                                <Suspense fallback=move || view! { <p style="font-size: 0.6rem;">"Loading..."</p> }>
                                                                    {move || url_res.get().flatten().map(|u| view! { <img src=u style="max-width: 100%; max-height: 100%; object-fit: contain;" /> })}
                                                                </Suspense>
                                                            }.into_view()
                                                        } else {
                                                            view! { <img src=img style="max-width: 100%; max-height: 100%; object-fit: contain;" /> }.into_view()
                                                        }
                                                    } else {
                                                        view! { <span style="font-size: 0.6rem; color: #ccc;">"ID BACK NOT SCANNED"</span> }.into_view()
                                                    }}
                                                </div>
                                                <p style="margin: 5px 0 0 0; font-size: 0.6rem; color: #95a5a6;">"Back Side"</p>
                                            </div>                                        </div>
                                        </div>                                </div>
                            }
                        } />
                    </div>

                    // Footer
                    <div style="margin-top: 50px; border-top: 1px solid #eee; padding-top: 20px; display: flex; justify-content: space-between; align-items: flex-end; page-break-inside: avoid; break-inside: avoid;">
                        <div style="font-size: 0.7rem; color: #95a5a6;">
                            <p style="margin: 0;">"System Generated Record"</p>
                            <p style="margin: 2px 0 0 0;">"Anand Lodge Management System"</p>
                        </div>
                        <div style="text-align: center; width: 200px;">
                            <div style="border-bottom: 1px solid #2c3e50; margin-bottom: 8px;"></div>
                            <p style="font-size: 0.7rem; color: #2c3e50; font-weight: 700; text-transform: uppercase;">"Manager Signature"</p>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                "@media print {
                    @page { margin: 0; size: A4; }
                    html, body { 
                        height: auto !important; 
                        overflow: visible !important; 
                        margin: 0 !important; 
                        padding: 0 !important; 
                        background: white !important;
                        -webkit-print-color-adjust: exact !important;
                        print-color-adjust: exact !important;
                    }

                    /* Hide everything except our overlay */
                    body * { visibility: hidden; }
                    .bill-overlay, .bill-overlay * { visibility: visible; }
                    .no-print, .sidebar, .mobile-header, .sidebar-overlay { display: none !important; }

                    .app-layout, .content, .container, .card, main { 
                        display: block !important; 
                        height: auto !important; 
                        overflow: visible !important; 
                        padding: 0 !important; 
                        margin: 0 !important; 
                        border: none !important; 
                        box-shadow: none !important;
                        background: transparent !important;
                        visibility: visible !important;
                    }

                    .bill-overlay { 
                        position: absolute !important; 
                        top: 0 !important; 
                        left: 0 !important; 
                        width: 100% !important; 
                        height: auto !important; 
                        background: white !important; 
                        padding: 0 !important; 
                        margin: 0 !important; 
                        z-index: 100000 !important;
                        overflow: visible !important;
                    }

                    .printable-area { 
                        border: none !important; 
                        box-shadow: none !important; 
                        width: 21cm !important; 
                        height: auto !important;
                        min-height: 29.7cm !important; 
                        padding: 1cm 1.5cm !important; 
                        margin: 0 auto !important; 
                        border-radius: 0 !important; 
                        display: block !important;
                    }
                    .watermark { display: none !important; }
                }"
            </style>        </div>
    }
}
