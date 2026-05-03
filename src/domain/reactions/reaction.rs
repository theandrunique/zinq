use chrono::{DateTime, Utc};

pub struct Reaction {
    pub chat_id: i64,
    pub message_id: i64,
    pub user_id: i64,
    pub pack_id: String,
    pub emoji_id: String,
    pub created_at: DateTime<Utc>,
}
