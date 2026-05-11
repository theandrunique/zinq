use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "metadata", rename_all = "snake_case")]
pub enum MessageType {
    Default,
    Reply { referenced_message_id: i64 },
    MemberAdd { user_id: i64 },
    MemberRemove { user_id: i64 },
    MemberLeave { user_id: i64 },
    ChatNameUpdate { new_name: String },
    ChatImageUpdate { new_image: String },
    ChatPinnedMessage,
    ChatUnpinMessage,
    ChatCreate { chat_name: String },
    Forward,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub author_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub message_type: MessageType,
}

pub struct CreateMessageRequest {
    pub id: i64,
    pub chat_id: i64,
    pub author_id: i64,
    pub content: String,
    pub message_type: MessageType,
}

impl Message {
    pub fn new(request: CreateMessageRequest) -> Self {
        Self {
            id: request.id,
            chat_id: request.chat_id,
            author_id: request.author_id,
            content: request.content,
            created_at: Utc::now(),
            edited_at: None,
            message_type: request.message_type,
        }
    }
}
