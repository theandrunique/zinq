use chrono::{DateTime, Utc};

pub struct MessageAck {
    pub chat_id: i64,
    pub user_id: i64,
    pub to_message_id: i64,
    pub from_message_id: i64,
    pub timestamp: DateTime<Utc>,
}
