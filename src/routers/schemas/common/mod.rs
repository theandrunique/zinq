use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::{
    auth::{SessionLifetime, User},
    chats::{Chat, ChatType},
};

#[derive(Serialize)]
pub struct UserPrivateSchema {
    pub id: String,
    pub username: String,
    pub global_name: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub timestamp: DateTime<Utc>,

    pub sessions_lifetime: SessionLifetime,
    pub mfa: bool,
    pub email: String,
    pub is_email_verified: bool,
}

impl From<User> for UserPrivateSchema {
    fn from(value: User) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            global_name: value.display_name,
            bio: value.bio,
            avatar: value.avatar,
            timestamp: value.created_at,
            sessions_lifetime: value.sessions_lifetime,
            mfa: value.mfa,
            email: value.email,
            is_email_verified: value.is_email_verified,
        }
    }
}

#[derive(Serialize)]
pub struct UserPublicSchema {
    pub id: String,
    pub username: String,
    pub global_name: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl From<User> for UserPublicSchema {
    fn from(value: User) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            global_name: value.display_name,
            bio: value.bio,
            avatar: value.avatar,
            timestamp: value.created_at,
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
    pub chat_type: ChatType,
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
        }
    }
}
