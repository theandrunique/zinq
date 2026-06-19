use std::sync::Arc;

use crate::{
    application::RequestHandler,
    core::ValidateExt,
    domain::{
        chats::{
            ChatPermissions, ChatType,
            data::{ChatLoadOptions, ChatLoader, ChatRepository},
        },
        events::{DomainEvent, Mediator},
    },
    error::Error,
    state::AppState,
};

#[derive(Debug, validator::Validate, Clone)]
pub struct DeleteChatMemberCommand {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub user_id: i64,
}

pub struct DeleteChatMemberCommandHandler {
    mediator: Arc<Mediator>,
    chat_repository: Arc<dyn ChatRepository>,
    chat_loader: Arc<dyn ChatLoader>,
}

impl DeleteChatMemberCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            mediator: Arc::clone(&state.mediator),
            chat_repository: Arc::clone(&state.chat_repository),
            chat_loader: Arc::clone(&state.chat_loader),
        }
    }
}

impl RequestHandler for DeleteChatMemberCommandHandler {
    type Request = DeleteChatMemberCommand;
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

        if chat.chat_type != ChatType::GroupDm {
            return Err(Error::ChatTypeNotSupported {
                chat_id: request.chat_id,
            });
        }

        if !chat.has_permission(request.current_user_id, ChatPermissions::MANAGE_MEMBERS) {
            return Err(Error::InsufficientPermissions {
                permission: ChatPermissions::MANAGE_MEMBERS,
                chat_id: request.chat_id,
            });
        }

        let member = chat.get_member(request.user_id);

        if let Some(member) = member {
            if member.is_leave {
                return Err(Error::UserNotMember {
                    user_id: request.user_id,
                    chat_id: request.chat_id,
                });
            }

            self.chat_repository
                .update_is_leave_status(request.user_id, request.chat_id, true)
                .await
                .map_err(Error::InternalServerError)?;

            self.mediator
                .publish(&DomainEvent::ChatMemberRemove {
                    member,
                    chat_id: chat.id,
                    initiator_id: request.current_user_id,
                })
                .await?;
        } else {
            return Err(Error::UserNotMember {
                user_id: request.user_id,
                chat_id: request.chat_id,
            });
        }

        Ok(())
    }
}
