use leptos::*;
use wasm_bindgen::prelude::*;
use crate::models::User;

pub fn is_bridge_ready() -> bool {
    if let Ok(ready) = js_sys::Reflect::get(&window(), &JsValue::from_str("bridgeReady")) {
        ready.as_bool().unwrap_or(false)
    } else { false }
}

pub async fn wait_for_bridge() {
    while !is_bridge_ready() { gloo_timers::future::TimeoutFuture::new(100).await; }
}

pub fn validate_aadhaar_checksum(aadhaar: &str) -> bool {
    let clean_aadhaar = aadhaar.trim().replace(" ", "").replace("-", "");
    if clean_aadhaar.len() != 12 || !clean_aadhaar.chars().all(|c| c.is_ascii_digit()) { return false; }
    let multiplication_table = [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],[1, 2, 3, 4, 0, 6, 7, 8, 9, 5],[2, 3, 4, 0, 1, 7, 8, 9, 5, 6],[3, 4, 0, 1, 2, 8, 9, 5, 6, 7],[4, 0, 1, 2, 3, 9, 5, 6, 7, 8],[5, 9, 8, 7, 6, 4, 3, 2, 1, 0],[6, 5, 9, 8, 7, 0, 4, 3, 2, 1],[7, 6, 5, 9, 8, 1, 0, 4, 3, 2],[8, 7, 6, 5, 9, 2, 1, 0, 4, 3],[9, 8, 7, 6, 5, 3, 2, 1, 0, 4]];
    let permutation_table = [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],[1, 5, 7, 6, 2, 8, 3, 0, 9, 4],[5, 8, 0, 3, 7, 9, 6, 1, 4, 2],[8, 9, 1, 6, 0, 4, 3, 5, 2, 7],[9, 4, 5, 3, 1, 2, 6, 8, 7, 0],[4, 2, 8, 6, 5, 7, 3, 9, 0, 1],[2, 7, 9, 3, 8, 0, 6, 4, 1, 5],[7, 0, 4, 6, 9, 1, 3, 2, 5, 8]];
    let mut c = 0;
    let digits: Vec<usize> = clean_aadhaar.chars().map(|d| d.to_digit(10).unwrap() as usize).collect();
    for (i, &digit) in digits.iter().rev().enumerate() { c = multiplication_table[c][permutation_table[i % 8][digit]]; }
    c == 0
}

pub fn calculate_age(dob: &str) -> Option<String> {
    let parts: Vec<&str> = dob.split('/').collect();
    if parts.len() == 3 {
        if let Ok(year) = parts[2].parse::<i32>() { return Some((2026 - year).to_string()); }
    }
    None
}

pub fn get_saved_user() -> Option<User> {
    let storage = window().local_storage().ok()??;
    let user_json = storage.get_item("user").ok()??;
    serde_json::from_str(&user_json).ok()
}

pub fn save_user(user: &User) {
    if let Ok(Some(s)) = window().local_storage() {
        let _ = s.set_item("user", &serde_json::to_string(user).unwrap_or_default());
    }
}

pub fn clear_user() {
    if let Ok(Some(s)) = window().local_storage() { let _ = s.remove_item("user"); }
}
