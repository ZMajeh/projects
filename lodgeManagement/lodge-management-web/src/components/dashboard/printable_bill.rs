use leptos::*;
use crate::models::{Booking, Customer};

#[component]
pub fn PrintableBill(booking: Booking, customer: Option<Customer>, on_close: Callback<()>) -> impl IntoView {
    let paid: f64 = booking.payments.iter().map(|p| p.amount).sum();
    let balance = booking.total_amount - paid;
    let today = js_sys::Date::new_0().to_iso_string().as_string().unwrap()[..10].to_string();
    
    view! {
        <div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: #555; z-index: 10000; overflow-y: auto; color: #000; padding: 20px 0;" class="bill-overlay">
            <div style="max-width: 800px; margin: 0 auto 15px auto; display: flex; justify-content: space-between; padding: 0 20px;" class="no-print">
                <button on:click=move |_| on_close.call(()) style="background: #e74c3c; padding: 10px 20px; border-radius: 4px; border: none; color: white; cursor: pointer; font-weight: bold;">"← Close"</button>
                <button on:click=move |_| { let _ = window().print(); } style="background: #2ecc71; padding: 10px 30px; font-weight: bold; border-radius: 4px; border: none; color: white; cursor: pointer; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">"PRINT INVOICE"</button>
            </div>

            <div style="width: 21cm; min-height: 29.7cm; margin: 0 auto; background: white; padding: 1cm 1.5cm; box-sizing: border-box; position: relative; box-shadow: 0 0 20px rgba(0,0,0,0.3); border-radius: 2px;" class="printable-area">
                // Background Watermark
                <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%) rotate(-45deg); font-size: 8rem; color: rgba(0,0,0,0.02); pointer-events: none; font-weight: 900; white-space: nowrap; z-index: 0;">"ANAND LODGE"</div>

                <div style="position: relative; z-index: 1;">
                    // Header
                    <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 30px; border-bottom: 4px double #2c3e50; padding-bottom: 15px;">
                        <div style="flex: 1;">
                            <h1 style="margin: 0; font-size: 2.4rem; color: #2c3e50; text-transform: uppercase; letter-spacing: 3px; font-weight: 900; line-height: 1;">"ANAND LODGE"</h1>
                            <h3 style="margin: 5px 0 0 0; font-size: 0.9rem; color: #7f8c8d; letter-spacing: 5px; text-transform: uppercase; font-weight: 400;">"LODGE & STAY"</h3>
                            <div style="margin-top: 15px; font-size: 0.8rem; color: #34495e; line-height: 1.5;">
                                <p style="margin: 0; display: flex; align-items: center;"><span style="width: 15px; margin-right: 8px;">"📍"</span> "Front of bus-stand, Gangakhed"</p>
                                <p style="margin: 0; display: flex; align-items: center;"><span style="width: 15px; margin-right: 8px;">"📞"</span> "Phone: +91 70660 58468"</p>
                                <p style="margin: 0; display: flex; align-items: center;"><span style="width: 15px; margin-right: 8px;">"✉️"</span> "Email: vijaymundhe90@gmail.com"</p>
                            </div>
                        </div>
                        <div style="width: 260px; margin-left: 20px;">
                            <div style="background: #2c3e50; color: white; padding: 10px; border-radius: 4px; margin-bottom: 15px; text-align: center;">
                                <h2 style="margin: 0; font-size: 1rem; letter-spacing: 1px; text-transform: uppercase; font-weight: 700;">"TAX INVOICE"</h2>
                            </div>
                            <table style="width: 100%; font-size: 0.85rem; color: #2c3e50; border-collapse: collapse;">
                                <tr>
                                    <td style="padding: 3px 0; font-weight: 700;">"Invoice Date:"</td>
                                    <td style="padding: 3px 0; text-align: right;">{today}</td>
                                </tr>
                                <tr>
                                    <td style="padding: 3px 0; font-weight: 700;">"Invoice No:"</td>
                                    <td style="padding: 3px 0; text-align: right; color: #e67e22; font-weight: 700;">"AL-" {booking.id.clone().unwrap_or_default().chars().take(8).collect::<String>().to_uppercase()}</td>
                                </tr>
                            </table>
                        </div>
                    </div>

                    // Guest and Stay Info
                    <div style="display: flex; gap: 20px; margin-bottom: 25px;">
                        <div style="flex: 1; border: 1px solid #eee; border-radius: 8px; overflow: hidden;">
                            <h4 style="margin: 0; background: #f8f9fa; color: #2c3e50; padding: 8px 12px; text-transform: uppercase; font-size: 0.7rem; letter-spacing: 1px; border-bottom: 1px solid #eee;">"Billed To"</h4>
                            <div style="padding: 10px 12px;">
                                <p style="margin: 0 0 4px 0; font-size: 1.1rem; color: #2c3e50; font-weight: 700;">{booking.customer_name.clone()}</p>
                                {if !booking.extra_guests.is_empty() {
                                    view! {
                                        <p style="margin: 0 0 6px 0; font-size: 0.75rem; color: #666; font-style: italic;">
                                            "Co-Guests: " {booking.extra_guests.iter().map(|g| g.name.clone()).collect::<Vec<_>>().join(", ")}
                                        </p>
                                    }.into_view()
                                } else { view! {}.into_view() }}
                                <p style="margin: 0; font-size: 0.8rem; color: #34495e;"><strong>"Contact: "</strong> {customer.as_ref().map(|c| c.phone.clone()).unwrap_or_else(|| "N/A".to_string())}</p>
                            </div>
                        </div>
                        <div style="flex: 1; border: 1px solid #eee; border-radius: 8px; overflow: hidden;">
                            <h4 style="margin: 0; background: #f8f9fa; color: #2c3e50; padding: 8px 12px; text-transform: uppercase; font-size: 0.7rem; letter-spacing: 1px; border-bottom: 1px solid #eee;">"Stay Details"</h4>
                            <div style="padding: 10px 12px;">
                                <table style="width: 100%; font-size: 0.8rem; color: #2c3e50; border-spacing: 0 4px;">
                                    <tr><td style="color: #7f8c8d;">"Room Number"</td><td style="text-align: right;"><strong>"Room " {booking.room_number.clone()}</strong></td></tr>
                                    <tr><td style="color: #7f8c8d;">"Arrival"</td><td style="text-align: right;">{format!("{} ({})", booking.check_in_date.clone(), booking.in_time.clone().unwrap_or_else(|| "--:--".to_string()))}</td></tr>
                                    <tr><td style="color: #7f8c8d;">"Departure"</td><td style="text-align: right;">{format!("{} ({})", booking.check_out_date.clone(), booking.out_time.clone().unwrap_or_else(|| "--:--".to_string()))}</td></tr>
                                </table>
                            </div>
                        </div>
                    </div>

                    // Items Table
                    <table style="width: 100%; border-collapse: collapse; margin-bottom: 25px;">
                        <thead>
                            <tr style="background: #2c3e50; color: white;">
                                <th style="text-align: left; padding: 10px 12px; font-size: 0.8rem; text-transform: uppercase;">"Service Description"</th>
                                <th style="text-align: right; padding: 10px 12px; font-size: 0.8rem; text-transform: uppercase; width: 150px;">"Amount"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr style="border-bottom: 1px solid #f0f0f0;">
                                <td style="padding: 15px 12px;">
                                    <div style="font-weight: 700; color: #2c3e50; font-size: 0.95rem; margin-bottom: 3px;">"Premium Room Accommodation"</div>
                                    <div style="color: #7f8c8d; font-size: 0.75rem;">"Includes all essential lodge services and amenities."</div>
                                </td>
                                <td style="text-align: right; padding: 15px 12px; font-weight: 700; color: #2c3e50; font-size: 1rem;">"₹" {format!("{:.2}", booking.total_amount)}</td>
                            </tr>
                        </tbody>
                    </table>

                    // Summary and Guidelines side-by-side
                    <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 30px; gap: 30px;">
                        <div style="flex: 1;">
                            <div style="background: #fff9f4; border-left: 4px solid #e67e22; padding: 12px; border-radius: 4px;">
                                <h5 style="margin: 0 0 6px 0; color: #e67e22; font-size: 0.75rem; text-transform: uppercase;">"Guidelines & Terms"</h5>
                                <ul style="margin: 0; padding: 0 0 0 12px; font-size: 0.65rem; color: #555; line-height: 1.4;">
                                    <li>"Standard check-out time is 11:00 AM."</li>
                                    <li>"Please return keys to reception upon departure."</li>
                                    <li>"Management is not responsible for loss of valuables."</li>
                                    <li>"Any damage to property will be charged to the bill."</li>
                                    <li>"Please turn off appliances when not in use."</li>
                                </ul>
                            </div>
                        </div>
                        <div style="width: 300px;">
                            <div style="background: #f8f9fa; padding: 12px; border-radius: 8px; border: 1px solid #eee;">
                                <table style="width: 100%; border-collapse: collapse; color: #2c3e50;">
                                    <tr>
                                        <td style="padding: 3px 0; font-size: 0.85rem;">"Sub Total"</td>
                                        <td style="padding: 3px 0; text-align: right; font-size: 0.85rem;">"₹" {format!("{:.2}", booking.total_amount)}</td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 3px 0; font-size: 0.85rem; color: #27ae60; font-weight: 600;">"Amount Paid"</td>
                                        <td style="padding: 3px 0; text-align: right; font-size: 0.85rem; color: #27ae60; font-weight: 600;">"- ₹" {format!("{:.2}", paid)}</td>
                                    </tr>
                                    <tr>
                                        <td colspan="2" style="padding: 6px 0;"><div style="height: 1px; background: #ddd;"></div></td>
                                    </tr>
                                    <tr>
                                        <td style="padding: 6px 0; font-weight: 700; font-size: 0.95rem;">"BALANCE DUE"</td>
                                        <td style="padding: 6px 0; text-align: right; font-weight: 900; font-size: 1.2rem; color: #e74c3c;">"₹" {format!("{:.2}", balance)}</td>
                                    </tr>
                                </table>
                            </div>
                        </div>
                    </div>

                    // Footer / Signatures
                    <div style="margin-top: 40px; display: flex; justify-content: space-between; align-items: flex-end;">
                        <div style="text-align: center; width: 170px;">
                            <div style="height: 30px;"></div>
                            <div style="border-bottom: 2px solid #2c3e50; margin-bottom: 8px;"></div>
                            <p style="font-size: 0.7rem; color: #2c3e50; font-weight: 600; text-transform: uppercase;">"Guest Signature"</p>
                        </div>
                        <div style="text-align: center; width: 170px;">
                            <div style="height: 30px; display: flex; align-items: center; justify-content: center;">
                                <p style="font-family: 'Brush Script MT', cursive; margin: 0; font-size: 1.5rem; color: #2c3e50;">"Anand"</p>
                            </div>
                            <div style="border-bottom: 2px solid #2c3e50; margin-bottom: 8px;"></div>
                            <p style="font-size: 0.7rem; color: #2c3e50; font-weight: 700; text-transform: uppercase;">"Authorized Signatory"</p>
                        </div>
                    </div>

                    // Bottom Message
                    <div style="margin-top: 30px; text-align: center; border-top: 1px solid #eee; padding-top: 10px;">
                        <p style="font-size: 0.8rem; color: #2c3e50; font-weight: 700; margin: 0;">"Thank you for choosing Anand Lodge!"</p>
                        <p style="font-size: 0.65rem; color: #7f8c8d; margin: 2px 0 0 0;">"We hope to see you again soon."</p>
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
                    }
                    .no-print, .sidebar, .mobile-header, .sidebar-overlay { display: none !important; }
                    .app-layout, .content, .container, .card { 
                        display: block !important; 
                        height: auto !important; 
                        overflow: visible !important; 
                        padding: 0 !important; 
                        margin: 0 !important; 
                        border: none !important; 
                        box-shadow: none !important;
                        background: transparent !important;
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
                    }
                }"
            </style>
        </div>
    }
}
