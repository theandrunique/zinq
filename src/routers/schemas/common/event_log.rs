use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    domain::event_log::{EventLog, EventLogType},
    routers::schemas::common::{ChatMemberSchema, ChatSchema, MessageSchema},
};

#[derive(Serialize)]
pub struct EventLogSchema {
    pub user_id: String,
    pub event_id: String,
    pub event_type: EventLogTypeSchema,
    pub created_at: DateTime<Utc>,
}

impl From<EventLog> for EventLogSchema {
    fn from(event_log: EventLog) -> Self {
        EventLogSchema {
            user_id: event_log.user_id.to_string(),
            event_id: event_log.event_id.to_string(),
            event_type: event_log.event_type.into(),
            created_at: event_log.created_at,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventLogTypeSchema {
    MessageCreate {
        message: MessageSchema,
    },
    MessageUpdate {
        message: MessageSchema,
    },
    MessageDelete {
        message_id: String,
    },
    ChatCreate {
        chat: ChatSchema,
    },
    ChatMemberAdded {
        chat_id: String,
        member: ChatMemberSchema,
    },
    ChatMemberRemoved {
        chat_id: String,
        member: ChatMemberSchema,
    },
}

impl From<EventLogType> for EventLogTypeSchema {
    fn from(event_type: EventLogType) -> Self {
        match event_type {
            EventLogType::MessageCreate {
                message,
                attachments,
            } => EventLogTypeSchema::MessageCreate {
                message: (message, attachments).into(),
            },
            EventLogType::MessageUpdate {
                message,
                attachments,
            } => EventLogTypeSchema::MessageUpdate {
                message: (message, attachments).into(),
            },
            EventLogType::MessageDelete { message_id } => EventLogTypeSchema::MessageDelete {
                message_id: message_id.to_string(),
            },
            EventLogType::ChatCreate { chat } => {
                EventLogTypeSchema::ChatCreate { chat: chat.into() }
            }
            EventLogType::ChatMemberAdded { chat_id, member } => {
                EventLogTypeSchema::ChatMemberAdded {
                    chat_id: chat_id.to_string(),
                    member: member.into(),
                }
            }
            EventLogType::ChatMemberRemoved { chat_id, member } => {
                EventLogTypeSchema::ChatMemberRemoved {
                    chat_id: chat_id.to_string(),
                    member: member.into(),
                }
            }
        }
    }
}
