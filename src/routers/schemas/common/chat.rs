use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::chats::{Chat, ChatMember, ChatType};

#[derive(Serialize)]
pub struct ChatMemberSchema {
    pub user_id: String,
    pub username: String,
    pub global_name: String,
    pub avatar: Option<String>,
    pub permissions: Option<String>,
}

impl From<ChatMember> for ChatMemberSchema {
    fn from(value: ChatMember) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            username: value.username,
            global_name: value.global_name,
            avatar: value.avatar,
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
    pub created_at: DateTime<Utc>,
    pub members: Vec<ChatMemberSchema>,

    pub last_read_message_id: Option<String>,
    pub max_read_message_id: Option<String>,
}

impl ChatSchema {
    pub fn from_chat_for_user(chat: Chat, current_user_id: i64) -> Self {
        let current_member = chat.get_member(current_user_id);

        if current_member.is_none() {
            tracing::warn!("Current user is not a member of this chat");
        }

        let current_last_read = current_member
            .and_then(|m| m.last_read_message_id)
            .map(|id| id.to_string());

        let max_read_message_id = chat
            .members
            .iter()
            .filter(|m| m.user_id != current_user_id)
            .filter_map(|m| m.last_read_message_id)
            .max();

        Self {
            id: chat.id.to_string(),
            owner_id: chat.owner_id.map(|id| id.to_string()),
            name: chat.name,
            description: Some("".to_string()),
            image: chat.image,
            chat_type: chat.chat_type,
            last_message_id: chat.last_message_id.map(|id| id.to_string()),
            permissions: chat.permissions.to_string(),
            created_at: chat.timestamp,
            members: chat.members.into_iter().map(|m| m.into()).collect(),
            last_read_message_id: current_last_read,
            max_read_message_id: max_read_message_id.map(|id| id.to_string()),
        }
    }
}

impl From<Chat> for ChatSchema {
    fn from(chat: Chat) -> Self {
        Self {
            id: chat.id.to_string(),
            owner_id: chat.owner_id.map(|id| id.to_string()),
            name: chat.name,
            description: Some("".to_string()),
            image: chat.image,
            chat_type: chat.chat_type,
            last_message_id: chat.last_message_id.map(|id| id.to_string()),
            permissions: chat.permissions.to_string(),
            created_at: chat.timestamp,
            members: chat.members.into_iter().map(|m| m.into()).collect(),
            last_read_message_id: None,
            max_read_message_id: None,
        }
    }
}
