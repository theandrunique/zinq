use chrono::DateTime;
use serde::Serialize;

use crate::domain::chats::{Chat, ChatMember, ChatType};

#[derive(Serialize)]
pub struct ChatMemberSchema {
    pub user_id: String,
    pub username: String,
    pub global_name: String,
    pub avatar: Option<String>,
    pub is_leave: bool,
    pub permissions: Option<String>,
}

impl From<ChatMember> for ChatMemberSchema {
    fn from(value: ChatMember) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            username: value.username,
            global_name: value.global_name,
            avatar: value.avatar,
            is_leave: value.is_leave,
            permissions: value.permissions.map(|p| p.to_string()),
        }
    }
}

#[derive(Serialize)]
pub struct ChatSchema {
    pub id: String,
    pub owner_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub last_message_id: Option<String>,
    pub permissions: String,
    pub created_at: DateTime<chrono::Utc>,
    pub members: Vec<ChatMemberSchema>,
}

impl From<Chat> for ChatSchema {
    fn from(value: Chat) -> Self {
        Self {
            id: value.id.to_string(),
            owner_id: value.owner_id.map(|id| id.to_string()),
            name: value.name,
            description: Some("".to_string()),
            image: value.image,
            chat_type: value.chat_type,
            last_message_id: value.last_message_id.map(|id| id.to_string()),
            permissions: value.permissions.to_string(),
            created_at: value.timestamp,
            members: value.members.into_iter().map(|m| m.into()).collect(),
        }
    }
}
