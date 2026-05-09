use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, client::session::Session};

use crate::domain::chats::data::{ChatLoadOptions, ChatLoader};
use crate::domain::chats::{Chat, ChatMember, ChatPermissions, ChatType};
use crate::infra::data::common::ScyllaCommon;

#[derive(Debug, DeserializeRow)]
struct ChatMemberDb {
    user_id: i64,
    chat_id: i64,
    last_read_message_id: Option<i64>,
    username: String,
    global_name: String,
    image: Option<String>,
    permission_overwrites: Option<i64>,
    is_leave: bool,
}

impl TryFrom<ChatMemberDb> for ChatMember {
    type Error = anyhow::Error;

    fn try_from(value: ChatMemberDb) -> Result<Self, Self::Error> {
        Ok(ChatMember {
            user_id: value.user_id,
            last_read_message_id: value.last_read_message_id,
            username: value.username,
            global_name: value.global_name,
            avatar: value.image,
            is_leave: value.is_leave,
            permissions: value
                .permission_overwrites
                .map(ChatPermissions::from_bits_truncate),
        })
    }
}

#[derive(Debug, DeserializeRow)]
struct ChatDb {
    chat_id: i64,
    #[scylla(rename = "type")]
    chat_type: i32,
    name: Option<String>,
    owner_id: Option<i64>,
    image: Option<String>,
    last_message_id: Option<i64>,
    permissions: i64,
    timestamp: DateTime<Utc>,
}

impl TryFrom<ChatDb> for Chat {
    type Error = anyhow::Error;

    fn try_from(value: ChatDb) -> Result<Self, Self::Error> {
        Ok(Chat {
            id: value.chat_id,
            owner_id: value.owner_id,
            name: value.name,
            image: value.image,
            chat_type: match value.chat_type {
                0 => ChatType::Dm,
                1 => ChatType::GroupDm,
                _ => return Err(anyhow::anyhow!("Unknown chat_type: {}", value.chat_type)),
            },
            last_message_id: value.last_message_id,
            timestamp: value.timestamp,
            permissions: ChatPermissions::from_bits_truncate(value.permissions),
            members: Vec::new(),
        })
    }
}

pub struct ScyllaChatLoader {
    session: Arc<Session>,
}

impl ScyllaChatLoader {
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl ChatLoader for ScyllaChatLoader {
    async fn load(&self, options: ChatLoadOptions) -> Result<Option<Chat>, anyhow::Error> {
        let chat_id = options
            .chat_id
            .ok_or_else(|| anyhow::anyhow!("chat_id is required"))?;
        let common = ScyllaCommon::new(self.session.clone());

        let chat_db: Option<ChatDb> = common
            .exec_first("SELECT * FROM chats_by_id WHERE chat_id = ?", (chat_id,))
            .await?;

        let chat_db = match chat_db {
            Some(c) => c,
            None => return Ok(None),
        };

        let members: Vec<ChatMember> = if options.member_ids.is_empty() {
            Vec::new()
        } else {
            let members_db: Vec<ChatMemberDb> = common
                .exec_all(
                    "SELECT * FROM chat_users_by_chat_id WHERE chat_id = ? AND user_id IN ?",
                    (chat_id, options.member_ids),
                )
                .await?;

            members_db
                .into_iter()
                .filter_map(|m| ChatMember::try_from(m).ok())
                .collect()
        };

        let mut chat = Chat::try_from(chat_db)?;
        chat.members = members;
        Ok(Some(chat))
    }
}
