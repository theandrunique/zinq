use std::sync::Arc;

use crate::{
    application::RequestHandler,
    core::ValidateExt,
    domain::{
        auth::data::user_repository::UserRepository,
        chats::{
            Chat, ChatMember, ChatPermissions, ChatType,
            data::ChatRepository,
            data::{ChatLoadOptions, ChatLoader},
        },
        events::{DomainEvent, EventBus},
    },
    error::Error,
    state::AppState,
};

#[derive(Debug, validator::Validate, Clone)]
pub struct AddChatMemberCommand {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub user_id: i64,
}

pub struct AddChatMemberCommandHandler {
    event_bus: Arc<EventBus>,
    user_repository: Arc<dyn UserRepository>,
    chat_repository: Arc<dyn ChatRepository>,
    chat_loader: Arc<dyn ChatLoader>,
}

impl AddChatMemberCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            event_bus: Arc::clone(&state.event_bus),
            user_repository: Arc::clone(&state.user_repository),
            chat_repository: Arc::clone(&state.chat_repotisory),
            chat_loader: Arc::clone(&state.chat_loader),
        }
    }
}

impl RequestHandler for AddChatMemberCommandHandler {
    type Request = AddChatMemberCommand;
    type Output = ();
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        request.validate()?;

        let options = ChatLoadOptions::default()
            .with_chat_id(request.chat_id)
            .with_member(request.current_user_id)
            .with_member(request.user_id);

        let chat = self
            .chat_loader
            .load(options)
            .await
            .map_err(Error::InternalServerError)?
            .ok_or_else(|| Error::ChatNotFound(request.chat_id))?;

        if !chat.has_member(request.current_user_id) {
            return Err(Error::UserNotMember {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
            });
        }

        if chat.chat_type != ChatType::GroupDM {
            return Err(Error::ChatTypeNotSupported {
                chat_id: request.chat_id,
            });
        }

        if !chat.has_permission(request.current_user_id, ChatPermissions::ADD_MEMBERS) {
            return Err(Error::InsufficientPermissions {
                permission: ChatPermissions::ADD_MEMBERS,
                chat_id: request.chat_id,
            });
        }

        let existing_member = chat.members.iter().find(|m| m.user_id == request.user_id);

        if let Some(member) = existing_member {
            if !member.is_leave {
                return Err(Error::UserAlreadyMember {
                    user_id: request.user_id,
                    chat_id: request.chat_id,
                });
            }

            let mut updated_member = member.clone();
            updated_member.set_leave_status(false);

            self.chat_repository
                .update_is_leave_status(request.user_id, request.chat_id, false)
                .await
                .map_err(Error::InternalServerError)?;

            self.event_bus.publish(DomainEvent::ChatMemberAdded {
                chat: chat.clone(),
                member: updated_member,
            });

            return Ok(());
        }

        let user = self
            .user_repository
            .get_by_id(request.user_id)
            .await
            .map_err(Error::InternalServerError)?
            .ok_or_else(|| Error::UserNotFound(request.user_id))?;

        let new_member = ChatMember::from(user);

        self.chat_repository
            .upsert_channel_member(request.chat_id, new_member.clone())
            .await
            .map_err(Error::InternalServerError)?;

        self.event_bus.publish(DomainEvent::ChatMemberAdded {
            chat: chat.clone(),
            member: new_member,
        });

        Ok(())
    }
}
