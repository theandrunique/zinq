use async_trait::async_trait;

use crate::domain::chats::Chat;

#[derive(Clone, Default)]
pub struct ChatLoadOptions {
    pub chat_id: Option<i64>,
    pub member_ids: Vec<i64>,
}

impl ChatLoadOptions {
    pub fn with_chat_id(mut self, chat_id: i64) -> Self {
        self.chat_id = Some(chat_id);
        self
    }

    pub fn with_member(mut self, member_id: i64) -> Self {
        self.member_ids.push(member_id);
        self
    }

    pub fn with_members(mut self, member_ids: Vec<i64>) -> Self {
        self.member_ids.extend(member_ids);
        self
    }
}

#[async_trait]
pub trait ChatLoader: Send + Sync {
    async fn load(&self, options: ChatLoadOptions) -> Result<Option<Chat>, anyhow::Error>;
}
