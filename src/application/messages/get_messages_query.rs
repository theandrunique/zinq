use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::{
        chats::data::{ChatLoadOptions, ChatLoader},
        messages::data::MessageRepository,
    },
    error::Error,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct GetMessagesQuery {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub before: Option<i64>,
    pub limit: i32,
}

pub struct GetMessagesQueryHandler {
    chat_loader: Arc<dyn ChatLoader>,
    message_repository: Arc<dyn MessageRepository>,
}

impl GetMessagesQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_loader: Arc::clone(&state.chat_loader),
            message_repository: Arc::clone(&state.message_repository),
        }
    }
}

impl RequestHandler for GetMessagesQueryHandler {
    type Request = GetMessagesQuery;
    type Output = Vec<crate::domain::messages::Message>;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let chat = self
            .chat_loader
            .load(
                ChatLoadOptions::default()
                    .with_chat_id(request.chat_id)
                    .with_member(request.current_user_id),
            )
            .await
            .map_err(|e| Error::InternalServerError(e))?
            .ok_or(Error::ChatNotFound(request.chat_id))?;

        if !chat.has_member(request.current_user_id) {
            return Err(Error::UserNotMember {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
            });
        }

        let before = request.before.unwrap_or(i64::MAX);

        let messages = self
            .message_repository
            .get_messages(request.chat_id, before, request.limit)
            .await
            .map_err(|e| Error::InternalServerError(e))?;

        Ok(messages)
    }
}
