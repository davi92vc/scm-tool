use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Device {
    pub id: Option<i64>,
    pub name: String,
    pub ip: String,
    pub is_active: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Check {
    pub id: Option<i64>,
    pub device_id: i64,
    pub timestamp: Option<String>,
    pub is_online: bool,
    pub latency_ms: Option<f64>,
    pub error_msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transition {
    pub id: Option<i64>,
    pub device_id: i64,
    pub from_status: String,
    pub to_status: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AppError {
    pub id: Option<i64>,
    pub source: String,
    pub message: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AppSettings {
    pub id: Option<i64>,
    pub online_interval_sec: i64,
    pub offline_interval_sec: i64,
    pub autostart_enabled: bool,
    pub updated_at: Option<String>,
}
