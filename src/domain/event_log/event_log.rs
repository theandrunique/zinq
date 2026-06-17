use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{
    attachments::Attachment,
    chats::{Chat, ChatMember},
    messages::Message,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum EventLogType {
    MessageCreate {
        message: Message,
        attachments: Vec<Attachment>,
    },
    MessageUpdate {
        message: Message,
        attachments: Vec<Attachment>,
    },
    MessageDelete {
        message_id: i64,
    },
    ChatCreate {
        chat: Chat,
    },
    ChatMemberAdded {
        chat_id: i64,
        member: ChatMember,
    },
    ChatMemberRemoved {
        chat_id: i64,
        member: ChatMember,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub user_id: i64,
    pub event_id: i64,
    pub event_type: EventLogType,
    pub created_at: DateTime<Utc>,
}
