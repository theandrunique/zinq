use async_trait::async_trait;

use crate::domain::chats::{Chat, ChatMember};

#[async_trait]
pub trait ChatRepository: Send + Sync {
    async fn upsert(&self, chat: Chat) -> Result<(), anyhow::Error>;
    async fn get_by_id(&self, chat_id: i64) -> Result<Option<Chat>, anyhow::Error>;
    async fn get_dm_channel(
        &self,
        user_id1: i64,
        user_id2: i64,
    ) -> Result<Option<Chat>, anyhow::Error>;
    async fn get_member_ids(&self, chat_id: i64) -> Result<Vec<(i64, bool)>, anyhow::Error>;
    async fn get_user_chats(&self, user_id: i64) -> Result<Vec<Chat>, anyhow::Error>;
    async fn upsert_channel_member(
        &self,
        chat_id: i64,
        member: ChatMember,
    ) -> Result<(), anyhow::Error>;
    async fn update_is_leave_status(
        &self,
        user_id: i64,
        chat_id: i64,
        is_leave: bool,
    ) -> Result<(), anyhow::Error>;
    async fn update_channel_info(&self, chat_id: i64) -> Result<(), anyhow::Error>;
    async fn update_owner_id(&self, chat_id: i64, owner_id: i64) -> Result<(), anyhow::Error>;
    async fn update_last_message_id(
        &self,
        chat_id: i64,
        last_message_id: Option<i64>,
    ) -> Result<(), anyhow::Error>;
}
