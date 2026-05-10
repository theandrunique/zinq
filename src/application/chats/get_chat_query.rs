use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::chats::{Chat, data::ChatRepository},
    error::Error,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct GetChatQuery {
    pub current_user_id: i64,
    pub chat_id: i64,
}

pub struct GetChatQueryHandler {
    chat_repository: Arc<dyn ChatRepository>,
}

impl GetChatQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_repository: Arc::clone(&state.chat_repository),
        }
    }
}

impl RequestHandler for GetChatQueryHandler {
    type Request = GetChatQuery;
    type Output = Chat;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let chat = self
            .chat_repository
            .get_by_id(request.chat_id)
            .await
            .map_err(|e| Error::InternalServerError(e))?
            .ok_or(Error::ChatNotFound(request.chat_id))?;

        if !chat.has_member(request.current_user_id) {
            return Err(Error::UserNotMember {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
            });
        }

        Ok(chat)
    }
}
