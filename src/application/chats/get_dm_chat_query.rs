use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::{
        auth::data::user_repository::UserRepository,
        chats::{Chat, ChatMember, data::ChatRepository},
    },
    error::Error,
    infra::IdGenerator,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct GetDMChatCommand {
    pub current_user_id: i64,
    pub user_id: i64,
}

pub struct GetDMChatCommandHandler {
    id_gen: Arc<dyn IdGenerator>,
    user_repository: Arc<dyn UserRepository>,
    chat_repository: Arc<dyn ChatRepository>,
}

impl GetDMChatCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            id_gen: Arc::clone(&state.id_gen),
            user_repository: Arc::clone(&state.user_repository),
            chat_repository: Arc::clone(&state.chat_repository),
        }
    }
}

impl RequestHandler for GetDMChatCommandHandler {
    type Request = GetDMChatCommand;
    type Output = Chat;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let existing_dm = self
            .chat_repository
            .get_dm_channel(request.current_user_id, request.user_id)
            .await
            .map_err(Error::InternalServerError)?;

        if let Some(dm) = existing_dm {
            return Ok(dm);
        }

        let user_ids_to_find = if request.current_user_id == request.user_id {
            vec![request.current_user_id]
        } else {
            vec![request.current_user_id, request.user_id]
        };

        let users = self
            .user_repository
            .get_by_ids(&user_ids_to_find)
            .await
            .map_err(Error::InternalServerError)?;

        if users.len() != user_ids_to_find.len() {
            let found_ids: Vec<i64> = users.iter().map(|u| u.id).collect();
            let missing_id = user_ids_to_find
                .iter()
                .find(|id| !found_ids.contains(id))
                .copied()
                .unwrap_or(0);
            return Err(Error::UserNotFound(missing_id));
        }

        let new_chat_id = self.id_gen.gen_id().await;
        let members: Vec<ChatMember> = users.into_iter().map(ChatMember::from).collect();
        let chat = Chat::create_dm(new_chat_id, members);

        self.chat_repository
            .save(&chat)
            .await
            .map_err(Error::InternalServerError)?;

        Ok(chat)
    }
}
