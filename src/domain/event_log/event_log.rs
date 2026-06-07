use chrono::{DateTime, Utc};

use crate::domain::{chats::Chat, messages::Message};

pub enum EventLogType {
    MessageCreate { message: Message },
    MessageUpdate { message: Message },
    MessageDelete { message_id: i64 },
    ChatAdd { chat: Chat },
}

pub struct EventLog {
    pub user_id: i64,
    pub event_id: i64,
    pub event_type: EventLogType,
    pub created_at: DateTime<Utc>,
}
