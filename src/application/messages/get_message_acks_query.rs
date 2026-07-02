use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::{
        chats::data::{ChatLoadOptions, ChatLoader},
        message_acks::{MessageAck, data::MessageAckRepository},
    },
    error::Error,
    state::AppState,
};

pub struct GetMessageAcksQuery {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub message_id: i64,
}

pub struct GetMessageAcksQueryHandler {
    chat_loader: Arc<dyn ChatLoader>,
    message_ack_repository: Arc<dyn MessageAckRepository>,
}

impl GetMessageAcksQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_loader: Arc::clone(&state.chat_loader),
            message_ack_repository: Arc::clone(&state.message_ack_repository),
        }
    }
}

impl RequestHandler for GetMessageAcksQueryHandler {
    type Request = GetMessageAcksQuery;
    type Output = Vec<MessageAck>;
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

        self.message_ack_repository
            .get_acks(request.chat_id, request.message_id)
            .await
            .map_err(Error::InternalServerError)
    }
}
