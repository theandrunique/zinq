use chrono::{DateTime, Utc};

pub struct MessageAck {
    pub chat_id: i64,
    pub message_id: i64,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
}
