use std::sync::Arc;

use serde::Serialize;

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
pub struct CountMessagesQuery {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub from_message_id: i64,
    pub to_message_id: i64,
}

pub struct CountMessagesQueryHandler {
    message_repository: Arc<dyn MessageRepository>,
    chat_loader: Arc<dyn ChatLoader>,
}

impl CountMessagesQueryHandler {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            message_repository: Arc::clone(&app_state.message_repository),
            chat_loader: Arc::clone(&app_state.chat_loader),
        }
    }
}

#[derive(Serialize)]
pub struct CountMessagesQueryResponse {
    pub count: i64,
}

impl RequestHandler for CountMessagesQueryHandler {
    type Request = CountMessagesQuery;
    type Output = CountMessagesQueryResponse;
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
            .map_err(Error::InternalServerError)?
            .ok_or(Error::ChatNotFound(request.chat_id))?;

        if !chat.has_member(request.current_user_id) {
            return Err(Error::UserNotMember {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
            });
        }

        let count = self
            .message_repository
            .count_messages(
                request.chat_id,
                request.from_message_id,
                request.to_message_id,
            )
            .await
            .map_err(Error::InternalServerError)?;

        Ok(CountMessagesQueryResponse { count })
    }
}
