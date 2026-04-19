use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = loginUser)]
    pub async fn login_user(email: String, pass: String) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = signOutUser)]
    pub async fn sign_out_user() -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = getRooms)]
    pub async fn get_rooms_js() -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = addRoom)]
    pub async fn add_room_js(room: JsValue) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = updateRoom)]
    pub async fn update_room_js(id: String, room: JsValue) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = deleteRoom)]
    pub async fn delete_room_js(id: String) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = getCustomers)]
    pub async fn get_customers_js(search: String) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = addCustomer)]
    pub async fn add_customer_js(customer: JsValue) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = updateCustomer)]
    pub async fn update_customer_js(id: String, customer: JsValue) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = deleteCustomer)]
    pub async fn delete_customer_js(id: String) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = getBookings)]
    pub async fn get_bookings_js() -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = addBooking)]
    pub async fn add_booking_js(booking: JsValue) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = startCamera)]
    pub async fn start_camera(id: String) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = takeSnapshot)]
    pub async fn take_snapshot(id: String) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = stopCamera)]
    pub async fn stop_camera() -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = extractAadhaar)]
    pub async fn extract_aadhaar_js(base64: String) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = readFileAsDataURL)]
    pub async fn read_file_as_data_url(file: web_sys::File) -> Result<JsValue, JsValue>;
    
    #[wasm_bindgen(catch, js_name = manualVerifyAadhaar)]
    pub async fn manual_verify_aadhaar(num: String) -> Result<JsValue, JsValue>;
}
