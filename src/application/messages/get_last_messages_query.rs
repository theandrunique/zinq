use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::{
        chats::data::ChatMemberRepository,
        messages::{Message, data::MessageRepository},
    },
    error::Error,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct GetLastMessagesQuery {
    pub current_user_id: i64,
    pub chat_ids: Vec<i64>,
}

pub struct GetLastMessagesQueryHandler {
    message_repository: Arc<dyn MessageRepository>,
    chat_member_repository: Arc<dyn ChatMemberRepository>,
}

impl GetLastMessagesQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            message_repository: Arc::clone(&state.message_repository),
            chat_member_repository: Arc::clone(&state.chat_member_repository),
        }
    }
}

impl RequestHandler for GetLastMessagesQueryHandler {
    type Request = GetLastMessagesQuery;
    type Output = Vec<Message>;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        if request.chat_ids.is_empty() {
            return Ok(vec![]);
        }

        let member_statuses = self
            .chat_member_repository
            .get_chat_ids_for_user(request.current_user_id, &request.chat_ids)
            .await
            .map_err(Error::InternalServerError)?;

        for &chat_id in &request.chat_ids {
            if member_statuses.get(&chat_id).copied() != Some(false) {
                return Err(Error::UserNotMember {
                    user_id: request.current_user_id,
                    chat_id,
                });
            }
        }

        let messages = self
            .message_repository
            .get_lasts_from(&request.chat_ids)
            .await
            .map_err(Error::InternalServerError)?;

        Ok(messages)
    }
}
