use async_trait::async_trait;

use crate::domain::messages::message::Message;

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn upsert(message: Message);
    async fn bulk_upsert(messages: Vec<Message>);
    async fn get_by_id(chat_id: i64, message_id: i64) -> Option<Message>;
    async fn get_by_ids(chat_id: i64, message_ids: Vec<i64>) -> Vec<Message>;
    async fn get_last_from(chat_ids: Vec<i64>) -> Vec<Message>;
    async fn get_messages(chat_id: i64, before: i64, limit: i32) -> Vec<Message>;
    async fn delete_by_id(chat_id: i64, message_id: i64);
}
