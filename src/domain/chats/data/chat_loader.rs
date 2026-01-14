use async_trait::async_trait;

use crate::domain::chats::chat::Chat;

#[async_trait]
pub trait ChatLoader: Send + Sync {
    fn with_id(chat_id: i64) -> Self;
    fn with_members(member_ids: Vec<i64>) -> Self;
    fn with_member(member_id: Vec<i64>) -> Self;
    async fn load() -> Option<Chat>;
}
