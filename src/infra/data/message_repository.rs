use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, client::session::Session};
use serde_json;

use crate::{
    domain::messages::{Message, MessageType, data::MessageRepository},
    infra::data::common::ScyllaCommon,
};

#[derive(Debug, DeserializeRow)]
struct MessageDb {
    chat_id: i64,
    message_id: i64,
    author_id: i64,
    content: String,
    timestamp: DateTime<Utc>,
    edited_timestamp: Option<DateTime<Utc>>,
    #[scylla(rename = "type")]
    message_type: String,
}

fn message_type_to_string(t: &MessageType) -> String {
    serde_json::to_string(t).unwrap_or_else(|_| "{\"type\":\"default\"}".to_string())
}

fn message_type_from_string(s: &str) -> MessageType {
    serde_json::from_str(s).unwrap_or(MessageType::Default)
}

impl TryFrom<MessageDb> for Message {
    type Error = anyhow::Error;

    fn try_from(value: MessageDb) -> Result<Self, Self::Error> {
        Ok(Message {
            id: value.message_id,
            chat_id: value.chat_id,
            author_id: value.author_id,
            content: value.content,
            created_at: value.timestamp,
            edited_at: value.edited_timestamp,
            message_type: message_type_from_string(&value.message_type),
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
            INSERT INTO messages_by_chat_id (
                chat_id,
                message_id,
                author_id,
                content,
                timestamp,
                edited_timestamp,
                type
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        ";

        self.common
            .exec(
                query,
                (
                    message.chat_id,
                    message.id,
                    message.author_id,
                    message.content,
                    message.created_at,
                    message.edited_at,
                    message_type_to_string(&message.message_type),
                ),
            )
            .await?;

        Ok(())
    }

    async fn bulk_upsert(&self, messages: &[Message]) -> Result<(), anyhow::Error> {
        for message in messages {
            self.upsert(message.clone()).await?;
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
            FROM messages_by_chat_id
            WHERE chat_id = ? AND message_id = ?
        ";

        let row: Option<MessageDb> = self.common.exec_first(query, (chat_id, message_id)).await?;

        row.map(Message::try_from).transpose()
    }

    async fn get_by_ids(
        &self,
        chat_id: i64,
        message_ids: &[i64],
    ) -> Result<Vec<Message>, anyhow::Error> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = "
            SELECT *
            FROM messages_by_chat_id
            WHERE chat_id = ? AND message_id IN ?
        ";

        let rows: Vec<MessageDb> = self.common.exec_all(query, (chat_id, message_ids)).await?;
        rows.into_iter().map(Message::try_from).collect()
    }

    async fn get_lasts_from(&self, chat_ids: &[i64]) -> Result<Vec<Message>, anyhow::Error> {
        let mut result = Vec::new();

        for chat_id in chat_ids {
            let query = "
                SELECT *
                FROM messages_by_chat_id
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
                FROM messages_by_chat_id
                WHERE chat_id = ?
                LIMIT ?
            ";
            self.common.exec_all(query, (chat_id, limit)).await?
        } else {
            let query = "
                SELECT *
                FROM messages_by_chat_id
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
        let query = "DELETE FROM messages_by_chat_id WHERE chat_id = ? AND message_id = ?";
        self.common.exec(query, (chat_id, message_id)).await?;
        Ok(())
    }
}
