use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{client::session::Session, DeserializeRow};

use crate::{
    domain::messages::{
        Message, MessageMetadata, MessageType, data::MessageRepository
    },
    infra::data::common::ScyllaCommon,
};

#[derive(Debug, DeserializeRow)]
struct MessageDb {
    chat_id: i64,
    message_id: i64,
    author_id: i64,
    target_user_id: Option<i64>,
    content: String,
    timestamp: DateTime<Utc>,
    edited_timestamp: Option<DateTime<Utc>>,
    #[scylla(rename = "type")]
    message_type: i32,
    referenced_message_id: Option<i64>,
}

fn message_type_to_i32(t: &MessageType) -> i32 {
    match t {
        MessageType::Default => 0,
        MessageType::Reply => 1,
        MessageType::MemberAdd => 2,
        MessageType::MemberRemove => 3,
        MessageType::MemberLeave => 4,
        MessageType::ChannelNameUpdate => 5,
        MessageType::ChannelImageUpdate => 6,
        MessageType::ChannelPinnedMessage => 7,
        MessageType::ChannelUnpinMessage => 8,
        MessageType::ChannelCreate => 9,
        MessageType::Forward => 10,
    }
}

fn message_type_from_i32(v: i32) -> MessageType {
    match v {
        1 => MessageType::Reply,
        2 => MessageType::MemberAdd,
        3 => MessageType::MemberRemove,
        4 => MessageType::MemberLeave,
        5 => MessageType::ChannelNameUpdate,
        6 => MessageType::ChannelImageUpdate,
        7 => MessageType::ChannelPinnedMessage,
        8 => MessageType::ChannelUnpinMessage,
        9 => MessageType::ChannelCreate,
        10 => MessageType::Forward,
        _ => MessageType::Default,
    }
}

impl TryFrom<MessageDb> for Message {
    type Error = anyhow::Error;

    fn try_from(value: MessageDb) -> Result<Self, Self::Error> {
        Ok(Message {
            id: value.message_id,
            channel_id: value.chat_id,
            author_id: value.author_id,
            target_user_id: value.target_user_id,
            content: value.content,
            timestamp: value.timestamp,
            edited_timestamp: value.edited_timestamp.unwrap_or(value.timestamp),
            message_type: message_type_from_i32(value.message_type),
            referenced_message_id: value.referenced_message_id,
            metadata: MessageMetadata::None,
        })
    }
}

pub struct ScyllaMessageRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaMessageRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: session.clone(),
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl MessageRepository for ScyllaMessageRepository {
    async fn upsert(&self, message: Message) -> Result<(), anyhow::Error> {
        let query = "
            INSERT INTO messages (
                chat_id,
                message_id,
                author_id,
                target_user_id,
                content,
                timestamp,
                edited_timestamp,
                pinned,
                type,
                referenced_message_id,
                metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ";

        self.common
            .exec(
                query,
                (
                    message.channel_id,
                    message.id,
                    message.author_id,
                    message.target_user_id,
                    message.content,
                    message.timestamp,
                    message.edited_timestamp,
                    false,                                      // pinned
                    message_type_to_i32(&message.message_type), // type
                    message.referenced_message_id,
                    Option::<String>::None, // metadata
                ),
            )
            .await?;

        Ok(())
    }

    async fn bulk_upsert(&self, messages: Vec<Message>) -> Result<(), anyhow::Error> {
        for message in messages {
            self.upsert(message).await?;
        }
        Ok(())
    }

    async fn get_by_id(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Option<Message>, anyhow::Error> {
        let query = "
            SELECT *
            FROM messages
            WHERE chat_id = ? AND message_id = ?
        ";

        let row: Option<MessageDb> =
            self.common.exec_first(query, (chat_id, message_id)).await?;

        row.map(Message::try_from).transpose()
    }

    async fn get_by_ids(
        &self,
        chat_id: i64,
        message_ids: Vec<i64>,
    ) -> Result<Vec<Message>, anyhow::Error> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = "
            SELECT *
            FROM messages
            WHERE chat_id = ? AND message_id IN ?
        ";

        let rows: Vec<MessageDb> = self.common.exec_all(query, (chat_id, message_ids)).await?;
        rows.into_iter().map(Message::try_from).collect()
    }

    async fn get_lasts_from(&self, chat_ids: Vec<i64>) -> Result<Vec<Message>, anyhow::Error> {
        let mut result = Vec::new();

        for chat_id in chat_ids {
            let query = "
                SELECT *
                FROM messages
                WHERE chat_id = ?
                LIMIT 1
            ";

            let row: Option<MessageDb> = self.common.exec_first(query, (chat_id,)).await?;
            if let Some(db) = row {
                result.push(Message::try_from(db)?);
            }
        }

        Ok(result)
    }

    async fn get_messages(
        &self,
        chat_id: i64,
        before: i64,
        limit: i32,
    ) -> Result<Vec<Message>, anyhow::Error> {
        let rows: Vec<MessageDb> = if before <= 0 {
            let query = "
                SELECT *
                FROM messages
                WHERE chat_id = ?
                LIMIT ?
            ";
            self.common.exec_all(query, (chat_id, limit)).await?
        } else {
            let query = "
                SELECT *
                FROM messages
                WHERE chat_id = ? AND message_id < ?
                LIMIT ?
            ";
            self.common
                .exec_all(query, (chat_id, before, limit))
                .await?
        };

        rows.into_iter().map(Message::try_from).collect()
    }

    async fn delete_by_id(&self, chat_id: i64, message_id: i64) -> Result<(), anyhow::Error> {
        let query = "DELETE FROM messages WHERE chat_id = ? AND message_id = ?";
        self.common.exec(query, (chat_id, message_id)).await?;
        Ok(())
    }
}
