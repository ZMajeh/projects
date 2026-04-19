use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub email: String,
    pub uid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Room {
    pub id: Option<String>,
    pub number: String,
    pub room_type: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct NewRoom {
    pub number: String,
    pub room_type: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
pub struct Booking {
    pub id: Option<String>,
    pub room_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub room_number: String,
    pub check_in_date: String,
    pub check_out_date: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct NewBooking {
    #[serde(rename = "roomId")]
    pub room_id: String,
    #[serde(rename = "customerId")]
    pub customer_id: String,
    #[serde(rename = "customerName")]
    pub customer_name: String,
    #[serde(rename = "roomNumber")]
    pub room_number: String,
    #[serde(rename = "checkInDate")]
    pub check_in_date: String,
    #[serde(rename = "checkOutDate")]
    pub check_out_date: String,
    pub status: String,
}
