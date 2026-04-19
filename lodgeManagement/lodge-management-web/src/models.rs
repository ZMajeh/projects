use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub email: String,
    pub uid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: Option<String>,
    pub number: String,
    pub room_type: String,
    pub status: String,
    #[serde(default)]
    pub price: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRoom {
    pub number: String,
    pub room_type: String,
    pub status: String,
    pub price: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub id: Option<String>,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub aadhaar: String,
    pub age: Option<String>,
    pub gender: Option<String>,
    pub photo_data: Option<String>,
    pub id_card_data: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCustomer {
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub aadhaar: String,
    pub age: Option<String>,
    pub gender: Option<String>,
    pub photo_data: Option<String>,
    pub id_card_data: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Payment {
    pub amount: f64,
    pub date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExtraGuest {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Booking {
    pub id: Option<String>,
    pub room_id: String,
    pub customer_id: String,
    pub customer_name: String,
    #[serde(default)]
    pub extra_guests: Vec<ExtraGuest>,
    pub room_number: String,
    pub check_in_date: String,
    pub check_out_date: String,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub status: String,
    #[serde(default)]
    pub total_amount: f64,
    #[serde(default)]
    pub payments: Vec<Payment>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBooking {
    pub room_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub extra_guests: Vec<ExtraGuest>,
    pub room_number: String,
    pub check_in_date: String,
    pub check_out_date: String,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub status: String,
    pub total_amount: f64,
    pub payments: Vec<Payment>,
}
