use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{chats::Chat, messages::Message};

#[derive(Clone, Serialize, Deserialize)]
pub enum EventLogType {
    MessageCreate { message: Message },
    MessageUpdate { message: Message },
    MessageDelete { message_id: i64 },
    ChatCreate { chat: Chat },
}

#[derive(Serialize, Deserialize)]
pub struct EventLog {
    pub user_id: i64,
    pub event_id: i64,
    pub event_type: EventLogType,
    pub created_at: DateTime<Utc>,
}
