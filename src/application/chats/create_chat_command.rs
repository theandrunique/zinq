use std::{collections::HashSet, sync::Arc};

use crate::{
    application::RequestHandler,
    core::ValidateExt,
    domain::{
        auth::data::user_repository::UserRepository,
        chats::{Chat, ChatMember, ChatPermissions, CreateGroupChatRequest, data::ChatRepository},
        events::{DomainEvent, EventBus},
    },
    error::Error,
    infra::IdGenerator,
    state::AppState,
};
use validator::ValidationError;

fn unique_members(members: &Vec<i64>) -> Result<(), ValidationError> {
    let mut seen = std::collections::HashSet::new();
    if members.iter().any(|id| !seen.insert(*id)) {
        return Err(ValidationError::new("members_must_be_unique")
            .with_message("members must contain unique ids".into()));
    }
    Ok(())
}

#[derive(Debug, validator::Validate, Clone)]
pub struct CreateChatCommand {
    pub current_user_id: i64,
    pub name: String,
    #[validate(custom(function = unique_members))]
    pub members: Vec<i64>,
    pub permissions: Option<ChatPermissions>,
}

pub struct CreateChatCommandHandler {
    id_gen: Arc<dyn IdGenerator>,
    event_bus: Arc<EventBus>,
    user_repository: Arc<dyn UserRepository>,
    chat_repository: Arc<dyn ChatRepository>,
}

impl CreateChatCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            id_gen: Arc::clone(&state.id_gen),
            event_bus: Arc::clone(&state.event_bus),
            user_repository: Arc::clone(&state.user_repository),
            chat_repository: Arc::clone(&state.chat_repository),
        }
    }
}

impl RequestHandler for CreateChatCommandHandler {
    type Request = CreateChatCommand;
    type Output = Chat;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        request.validate()?;
        let mut request = request;

        if !request.members.contains(&request.current_user_id) {
            request.members.push(request.current_user_id);
        }

        let users = self
            .user_repository
            .get_by_ids(&request.members)
            .await
            .map_err(|e| Error::InternalServerError(e))?;

        let found_ids: HashSet<i64> = users.iter().map(|u| u.id).collect();
        let requested_ids: HashSet<i64> = request.members.iter().cloned().collect();

        let missing: Vec<i64> = requested_ids.difference(&found_ids).copied().collect();

        if !missing.is_empty() {
            return Err(Error::UsersNotFound(missing));
        }

        let new_chat = Chat::create_group_dm(CreateGroupChatRequest {
            id: self.id_gen.gen_id().await,
            owner_id: request.current_user_id,
            name: request.name,
            members: users.into_iter().map(|u| ChatMember::from(u)).collect(),
            permissions: request.permissions,
        });

        self.chat_repository.save(new_chat.clone()).await?;

        self.event_bus.publish(DomainEvent::ChatCreate {
            chat: new_chat.clone(),
        });

        Ok(new_chat)
    }
}
