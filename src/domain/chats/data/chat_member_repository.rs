use std::collections::HashMap;

use async_trait::async_trait;

#[async_trait]
pub trait ChatMemberRepository: Send + Sync {
    async fn get_chat_ids_for_user(
        &self,
        user_id: i64,
        chat_ids: &[i64],
    ) -> Result<HashMap<i64, bool>, anyhow::Error>;
}
