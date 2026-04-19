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
pub struct Payment {
    pub amount: f64,
    pub date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Booking {
    pub id: Option<String>,
    pub room_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub room_number: String,
    pub check_in_date: String,
    pub check_out_date: String,
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
    pub room_number: String,
    pub check_in_date: String,
    pub check_out_date: String,
    pub status: String,
    pub total_amount: f64,
    pub payments: Vec<Payment>,
}
