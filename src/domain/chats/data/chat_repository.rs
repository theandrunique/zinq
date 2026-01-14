use async_trait::async_trait;

use crate::domain::chats::{chat::Chat, chat_member::ChatMember};

#[async_trait]
pub trait ChatRepository: Send + Sync {
    async fn upsert(chat: Chat);
    async fn get_by_id(chat_id: i64) -> Option<Chat>;
    async fn get_dm_channel(user_id1: i64, user_id2: i64) -> Vec<Chat>;
    async fn get_member_ids(chat_id: i64) -> Vec<(i64, bool)>;
    async fn get_user_chats(user_id: i64) -> Vec<Chat>;
    async fn upsert_channel_member(chat_id: i64, member: ChatMember);
    async fn update_is_leave_status(user_id: i64, chat_id: i64, is_leave: bool);
    async fn update_channel_info(chat_id: i64);
    async fn update_owner_id(chat_id: i64, owner_id: i64);
    async fn update_last_message_id(chat_id: i64, last_message_id: Option<i64>);
}
