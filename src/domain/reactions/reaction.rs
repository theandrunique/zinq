use chrono::{DateTime, Utc};

pub struct Reaction {
    pub message_id: i64,
    pub user_id: i64,
    pub reaction_type: String,
    pub emoji: String,
    pub timestamp: DateTime<Utc>
}
