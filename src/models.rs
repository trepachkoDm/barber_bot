use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Users {
    pub user_id: Uuid,
    pub telegram_id: i64,
    pub user_name: String,
    pub user_contact_info: String,
    pub user_visit_count: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Services {
    pub service_id: i64,
    pub service_name: String,
    pub service_duration: Option<i64>,
    pub service_price: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Appointments {
    pub appointment_id: i64,
    pub user_id: Uuid,
    pub user_name: String,
    pub service_name: String,
    pub service_id: i64,
    pub appointment_time: String,
    pub status: String,
}
