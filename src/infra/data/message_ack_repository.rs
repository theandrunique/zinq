use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, SerializeRow, client::session::Session};

use crate::{
    domain::message_acks::{MessageAck, data::MessageAckRepository},
    infra::data::common::ScyllaCommon,
};

#[derive(SerializeRow, DeserializeRow)]
struct MessageAckDb {
    chat_id: i64,
    message_id: i64,
    user_id: i64,
    timestamp: DateTime<Utc>,
}

impl From<MessageAckDb> for MessageAck {
    fn from(value: MessageAckDb) -> Self {
        MessageAck {
            chat_id: value.chat_id,
            message_id: value.message_id,
            user_id: value.user_id,
            created_at: value.timestamp,
        }
    }
}

pub struct ScyllaMessageAckRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaMessageAckRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: Arc::clone(&session),
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl MessageAckRepository for ScyllaMessageAckRepository {
    async fn upsert(&self, message_ack: &MessageAck) -> Result<(), anyhow::Error> {
        let query = "
            INSERT INTO message_acks (
                chat_id,
                message_id,
                user_id,
                timestamp
            ) VALUES (?, ?, ?, ?)
        ";

        self.common
            .exec(
                query,
                (
                    message_ack.chat_id,
                    message_ack.message_id,
                    message_ack.user_id,
                    message_ack.created_at,
                ),
            )
            .await?;
        Ok(())
    }

    async fn bulk_upsert(&self, message_acks: &[MessageAck]) -> Result<(), anyhow::Error> {
        for ack in message_acks {
            self.upsert(ack).await?;
        }
        Ok(())
    }

    async fn get_acks(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Vec<MessageAck>, anyhow::Error> {
        let query = "
            SELECT *
            FROM message_acks
            WHERE chat_id = ? AND message_id = ?
        ";

        let rows: Vec<MessageAckDb> = self.common.exec_all(query, (chat_id, message_id)).await?;

        Ok(rows.into_iter().map(MessageAck::from).collect())
    }
}
