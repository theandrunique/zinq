use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use scylla::client::session::Session;

use crate::domain::chats::data::ChatMemberRepository;
use crate::infra::data::common::ScyllaCommon;

#[derive(Debug, scylla::DeserializeRow)]
struct ChatMemberStatus {
    chat_id: i64,
    is_leave: bool,
}

pub struct ScyllaChatMemberRepository {
    common: ScyllaCommon,
}

impl ScyllaChatMemberRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl ChatMemberRepository for ScyllaChatMemberRepository {
    async fn get_chat_ids_for_user(
        &self,
        user_id: i64,
        chat_ids: &[i64],
    ) -> Result<HashMap<i64, bool>, anyhow::Error> {
        if chat_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let query = "
            SELECT chat_id, is_leave
            FROM chat_users_by_user_id
            WHERE user_id = ? AND chat_id = ?
        ";

        let mut result = HashMap::new();
        for &chat_id in chat_ids {
            let rows: Vec<ChatMemberStatus> =
                self.common.exec_all(query, (user_id, chat_id)).await?;
            for row in rows {
                result.insert(row.chat_id, !row.is_leave);
            }
        }

        Ok(result)
    }
}
