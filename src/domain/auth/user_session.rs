use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub token_id: Uuid,
    pub device_name: String,
    pub client_name: String,
    pub location: String,
    pub last_refresh_timestamp: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
}

impl UserSession {
    pub fn create(
        id: i64,
        user_id: i64,
        device_name: String,
        client_name: String,
        location: String,
    ) -> Self {
        let timestamp = Utc::now();
        Self {
            id: id,
            user_id: user_id,
            token_id: Uuid::new_v4(),
            device_name: device_name,
            client_name: client_name,
            location: location,
            last_refresh_timestamp: timestamp,
            timestamp: timestamp,
        }
    }
}
