use async_trait::async_trait;

use crate::domain::chats::Chat;

pub trait ChatLoaderFactory {
    fn create() -> impl ChatLoader;
}

#[async_trait]
pub trait ChatLoader: Send + Sync {
    fn with_id(&self, chat_id: i64) -> Self;
    fn with_members(&self, member_ids: Vec<i64>) -> Self;
    fn with_member(&self, member_id: i64) -> Self;
    async fn load(&self) -> Option<Chat>;
}
