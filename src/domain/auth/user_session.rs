use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub token_id: Uuid,
    pub device_name: String,
    pub client_name: String,
    pub location: String,
    pub last_refresh_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub struct UserSessionCreateRequest {
    pub id: i64,
    pub user_id: i64,
    pub device_name: String,
    pub client_name: String,
    pub location: String,
}

impl UserSession {
    pub fn create(request: UserSessionCreateRequest) -> Self {
        let current_time = Utc::now();

        Self {
            id: request.id,
            user_id: request.user_id,
            token_id: Uuid::new_v4(),
            device_name: request.device_name,
            client_name: request.client_name,
            location: request.location,
            last_refresh_at: current_time,
            created_at: current_time,
        }
    }
}
