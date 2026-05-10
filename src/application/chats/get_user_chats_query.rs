use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::chats::{Chat, ChatType, data::ChatRepository},
    error::Error,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct GetUserChatsQuery {
    pub current_user_id: i64,
}

pub struct GetUserChatsQueryHandler {
    chat_repository: Arc<dyn ChatRepository>,
}

impl GetUserChatsQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_repository: Arc::clone(&state.chat_repository),
        }
    }
}

impl RequestHandler for GetUserChatsQueryHandler {
    type Request = GetUserChatsQuery;
    type Output = Vec<Chat>;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let mut chats = self
            .chat_repository
            .get_user_chats(request.current_user_id)
            .await
            .map_err(|e| Error::InternalServerError(e))?;

        chats.retain(|chat| {
            let is_dm_without_message =
                chat.chat_type == ChatType::Dm && chat.last_message_id.is_none();
            !is_dm_without_message
        });

        Ok(chats)
    }
}
