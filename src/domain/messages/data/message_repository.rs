use async_trait::async_trait;

use crate::domain::messages::Message;

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn upsert(&self, message: Message) -> Result<(), anyhow::Error>;
    async fn bulk_upsert(&self, messages: &[Message]) -> Result<(), anyhow::Error>;

    async fn get_by_id(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Option<Message>, anyhow::Error>;

    async fn get_by_ids(
        &self,
        chat_id: i64,
        message_ids: &[i64],
    ) -> Result<Vec<Message>, anyhow::Error>;

    async fn get_lasts_from(&self, chat_ids: &[i64]) -> Result<Vec<Message>, anyhow::Error>;
    async fn get_messages(
        &self,
        chat_id: i64,
        before: i64,
        limit: i32,
    ) -> Result<Vec<Message>, anyhow::Error>;

    async fn delete_by_id(&self, chat_id: i64, message_id: i64) -> Result<(), anyhow::Error>;
}
