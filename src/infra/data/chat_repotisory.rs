use std::{collections::HashMap, str::FromStr, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, client::session::Session};

use crate::{
    domain::chats::{Chat, ChatMember, ChatPermissions, ChatType, data::ChatRepository},
    infra::data::common::ScyllaCommon,
};

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
                .map(ChatPermissions::try_from)
                .transpose()?,
        })
    }
}

#[derive(Debug, DeserializeRow)]
struct ChatDb {
    chat_id: i64,
    #[scylla(rename = "type")]
    chat_type: String,
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
            chat_type: ChatType::from_str(&value.chat_type).map_err(|e| {
                anyhow::anyhow!("Error parsing chat_type '{}': {}", value.chat_type, e)
            })?,
            last_message_id: value.last_message_id,
            timestamp: value.timestamp,
            permissions: ChatPermissions::try_from(value.permissions)?,
            members: Vec::new(),
        })
    }
}

impl TryFrom<(ChatDb, Vec<ChatMember>)> for Chat {
    type Error = anyhow::Error;

    fn try_from((db, members): (ChatDb, Vec<ChatMember>)) -> Result<Self, Self::Error> {
        Ok(Chat {
            id: db.chat_id,
            owner_id: db.owner_id,
            name: db.name,
            image: db.image,
            chat_type: ChatType::from_str(&db.chat_type).map_err(|e| {
                anyhow::anyhow!("Error parsing chat_type '{}': {}", db.chat_type, e)
            })?,
            last_message_id: db.last_message_id,
            timestamp: db.timestamp,
            permissions: ChatPermissions::try_from(db.permissions)?,
            members: members,
        })
    }
}

pub struct ScyllaChatRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaChatRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: session.clone(),
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl ChatRepository for ScyllaChatRepository {
    async fn upsert(&self, chat: Chat) -> Result<(), anyhow::Error> {
        // Insert chat row
        let query_chat = "
            INSERT INTO chats_by_id (
                chat_id,
                type,
                name,
                owner_id,
                image,
                last_message_id,
                permissions,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        self.common
            .exec(
                query_chat,
                (
                    chat.id,
                    chat.chat_type.to_string(),
                    chat.name.clone(),
                    chat.owner_id,
                    chat.image.clone(),
                    chat.last_message_id,
                    chat.permissions.bits(),
                    chat.timestamp,
                ),
            )
            .await?;

        // For DM chats also maintain mapping in private_chats
        if chat.chat_type == ChatType::DM && chat.members.len() == 2 {
            let user_id1 = chat.members[0].user_id;
            let user_id2 = chat.members[1].user_id;
            let (u1, u2) = if user_id1 < user_id2 {
                (user_id1, user_id2)
            } else {
                (user_id2, user_id1)
            };

            let query_private = "
                INSERT INTO private_chats (
                    user_id1,
                    user_id2,
                    chat_id
                ) VALUES (?, ?, ?)
            ";

            self.common.exec(query_private, (u1, u2, chat.id)).await?;
        }

        // Insert members into chat_users_by_user_id
        let query_member = "
            INSERT INTO chat_users_by_user_id (
                user_id,
                chat_id,
                last_read_message_id,
                username,
                global_name,
                image,
                permission_overwrites,
                is_leave
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        for member in &chat.members {
            self.common
                .exec(
                    query_member,
                    (
                        member.user_id,
                        chat.id,
                        member.last_read_message_id,
                        member.username.clone(),
                        member.global_name.clone(),
                        member.avatar.clone(),
                        member.permissions.as_ref().map(|p| p.bits()),
                        member.is_leave,
                    ),
                )
                .await?;
        }

        Ok(())
    }

    async fn get_by_id(&self, chat_id: i64) -> Result<Option<Chat>, anyhow::Error> {
        let result: Option<ChatDb> = self
            .common
            .exec_first("SELECT * FROM chats_by_id WHERE chat_id = ?", (chat_id,))
            .await?;

        result.map(Chat::try_from).transpose()
    }

    async fn get_dm_channel(
        &self,
        user_id1: i64,
        user_id2: i64,
    ) -> Result<Option<Chat>, anyhow::Error> {
        let values = if user_id1 < user_id2 {
            (user_id1, user_id2)
        } else {
            (user_id2, user_id1)
        };

        let result: Option<(i64,)> = self
            .common
            .exec_first(
                "SELECT chat_id from private_chats WHERE user_id1 = ? AND user_id2 = ?",
                values,
            )
            .await?;

        match result {
            Some(result) => self.get_by_id(result.0).await,
            None => Ok(None),
        }
    }

    async fn get_member_ids(&self, chat_id: i64) -> Result<Vec<(i64, bool)>, anyhow::Error> {
        let query =
            "SELECT user_id, is_leave FROM chat_users_by_chat_id WHERE chat_id = ?";

        let result: Vec<(i64, bool)> = self.common.exec_all(query, (chat_id,)).await?;

        Ok(result)
    }

    async fn get_user_chats(&self, user_id: i64) -> Result<Vec<Chat>, anyhow::Error> {
        let query = "SELECT chat_id FROM chat_users_by_user_id WHERE user_id = ?";
        let result: Vec<(i64,)> = self.common.exec_all(query, (user_id,)).await?;
        let chat_ids: Vec<i64> = result.into_iter().map(|v| v.0).collect();

        if chat_ids.is_empty() {
            return Ok(vec![]);
        }

        let query = "SELECT * FROM chats_by_id WHERE chat_id IN ?";
        let chats_db: Vec<ChatDb> = self.common.exec_all(query, (chat_ids.clone(),)).await?;

        let query = "SELECT * FROM chat_users_by_chat_id WHERE chat_id IN ?";
        let members_db: Vec<ChatMemberDb> = self.common.exec_all(query, (chat_ids,)).await?;

        let mut members_by_chat: HashMap<i64, Vec<ChatMember>> = HashMap::new();

        for member in members_db {
            members_by_chat
                .entry(member.chat_id)
                .or_default()
                .push(ChatMember::try_from(member)?);
        }

        let chats: Vec<Chat> = chats_db
            .into_iter()
            .map(|chat_db| {
                let members = members_by_chat.remove(&chat_db.chat_id).unwrap_or_default();
                Chat::try_from((chat_db, members))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(chats)
    }

    async fn upsert_channel_member(
        &self,
        chat_id: i64,
        member: ChatMember,
    ) -> Result<(), anyhow::Error> {
        let query = "
            INSERT INTO chat_users_by_user_id (
                user_id,
                chat_id,
                last_read_message_id,
                username,
                global_name,
                image,
                permission_overwrites,
                is_leave
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";
        self.common
            .exec(
                query,
                (
                    member.user_id,
                    chat_id,
                    member.last_read_message_id,
                    member.username,
                    member.global_name,
                    member.avatar,
                    member.permissions.map(|v| v.bits()),
                    member.is_leave,
                ),
            )
            .await?;
        Ok(())
    }

    async fn update_is_leave_status(
        &self,
        user_id: i64,
        chat_id: i64,
        is_leave: bool,
    ) -> Result<(), anyhow::Error> {
        let query =
            "UPDATE chat_users_by_user_id SET is_leave = ? WHERE user_id = ? AND chat_id = ?";
        self.common
            .exec(query, (is_leave, user_id, chat_id))
            .await?;
        Ok(())
    }

    async fn update_channel_info(&self, chat_id: i64) -> Result<(), anyhow::Error> {
        // There is no additional data passed here, so for now this is a no-op.
        // This can be extended later to update denormalized channel information.
        let _ = chat_id;
        Ok(())
    }

    async fn update_owner_id(&self, chat_id: i64, owner_id: i64) -> Result<(), anyhow::Error> {
        let query = "UPDATE chats_by_id SET owner_id = ? WHERE chat_id = ?";
        self.common.exec(query, (owner_id, chat_id)).await?;
        Ok(())
    }

    async fn update_last_message_id(
        &self,
        chat_id: i64,
        last_message_id: Option<i64>,
    ) -> Result<(), anyhow::Error> {
        let query = "UPDATE chats_by_id SET last_message_id = ? WHERE chat_id = ?";
        self.common.exec(query, (last_message_id, chat_id)).await?;
        Ok(())
    }
}
