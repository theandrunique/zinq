use std::sync::Arc;

use chrono::Utc;

use crate::{
    application::{RequestHandler, services::AttachmentService},
    core::ValidateExt,
    domain::{
        attachments::{Attachment, data::AttachmentRepository},
        chats::{
            ChatPermissions, ChatType,
            data::{ChatLoadOptions, ChatLoader},
        },
        events::DomainEvent,
        messages::{CreateMessageRequest, Message, MessageType, data::MessageRepository},
    },
    error::Error,
    state::AppState,
};

#[derive(Debug, validator::Validate, Clone)]
pub struct MessageAttachmentInput {
    pub uploaded_filename: String,
    pub filename: String,
}

#[derive(Debug, validator::Validate, Clone)]
pub struct AddOrEditMessageCommand {
    pub current_user_id: i64,
    pub message_id: Option<i64>,
    pub referenced_message_id: Option<i64>,
    pub chat_id: i64,
    pub content: String,
    pub attachments: Vec<MessageAttachmentInput>,
}

#[derive(Debug, Clone)]
pub struct AddOrEditMessageCommandResult {
    pub message: Message,
    pub attachments: Vec<Attachment>,
}

pub struct AddOrEditMessageCommandHandler {
    chat_loader: Arc<dyn ChatLoader>,
    message_repository: Arc<dyn MessageRepository>,
    attachment_repository: Arc<dyn AttachmentRepository>,
    attachment_service: Arc<AttachmentService>,
    id_gen: Arc<dyn crate::infra::IdGenerator>,
    event_bus: Arc<crate::domain::events::EventBus>,
}

impl AddOrEditMessageCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_loader: Arc::clone(&state.chat_loader),
            message_repository: Arc::clone(&state.message_repository),
            attachment_repository: Arc::clone(&state.attachment_repository),
            attachment_service: Arc::clone(&state.attachment_service),
            id_gen: Arc::clone(&state.id_gen),
            event_bus: Arc::clone(&state.event_bus),
        }
    }
}

impl RequestHandler for AddOrEditMessageCommandHandler {
    type Request = AddOrEditMessageCommand;
    type Output = AddOrEditMessageCommandResult;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        request.validate()?;

        let options = ChatLoadOptions::default()
            .with_chat_id(request.chat_id)
            .with_member(request.current_user_id);

        let chat = self
            .chat_loader
            .load(options)
            .await
            .map_err(Error::InternalServerError)?
            .ok_or(Error::ChatNotFound(request.chat_id))?;

        let initiator = chat
            .get_member(request.current_user_id)
            .ok_or(Error::UserNotMember {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
            })?;

        if !chat.has_permission(request.current_user_id, ChatPermissions::SEND_MESSAGES) {
            return Err(Error::InsufficientPermissions {
                permission: ChatPermissions::SEND_MESSAGES,
                chat_id: request.chat_id,
            });
        }

        let message_id = request.message_id.unwrap_or(self.id_gen.gen_id().await);

        let mut attachments: Vec<Attachment> = Vec::new();
        if !request.attachments.is_empty() {
            if !chat.has_permission(request.current_user_id, ChatPermissions::SEND_FILES) {
                return Err(Error::InsufficientPermissions {
                    permission: ChatPermissions::SEND_FILES,
                    chat_id: request.chat_id,
                });
            }

            for attachment_input in request.attachments {
                let attachment = self
                    .attachment_service
                    .validate_and_create_attachment(
                        message_id,
                        &attachment_input.uploaded_filename,
                        &attachment_input.filename,
                    )
                    .await?;
                attachments.push(attachment);
            }
        }

        let mut referenced_message: Option<Message> = None;
        if let Some(ref_msg_id) = request.referenced_message_id {
            let msg = self
                .message_repository
                .get_by_id(request.chat_id, ref_msg_id)
                .await
                .map_err(Error::InternalServerError)?
                .ok_or(Error::MessageNotFound(ref_msg_id))?;
            referenced_message = Some(msg);
        }

        if request.message_id.is_some() {
            let mut message = self
                .message_repository
                .get_by_id(request.chat_id, message_id)
                .await
                .map_err(Error::InternalServerError)?
                .ok_or(Error::MessageNotFound(message_id))?;

            if message.author_id != request.current_user_id {
                return Err(Error::MessageWasSentByAnotherUser(message_id));
            }

            message.content = request.content;
            message.edited_at = Some(Utc::now());

            self.message_repository
                .upsert(message.clone())
                .await
                .map_err(Error::InternalServerError)?;

            self.attachment_repository
                .bulk_save(&attachments)
                .await
                .map_err(Error::InternalServerError)?;

            self.event_bus.publish(DomainEvent::MessageUpdated {
                chat: chat.clone(),
                message: message.clone(),
                member: initiator.clone(),
            });

            Ok(AddOrEditMessageCommandResult {
                message,
                attachments,
            })
        } else {
            let message_id = self.id_gen.gen_id().await;
            let message_type = if request.referenced_message_id.is_some() {
                MessageType::Reply { referenced_message_id: request.referenced_message_id.unwrap() }
            } else {
                MessageType::Default
            };

            let message = Message::new(CreateMessageRequest {
                id: message_id,
                chat_id: request.chat_id,
                author_id: request.current_user_id,
                content: request.content,
                message_type,
            });

            self.message_repository
                .upsert(message.clone())
                .await
                .map_err(Error::InternalServerError)?;

            self.attachment_repository
                .bulk_save(&attachments)
                .await
                .map_err(Error::InternalServerError)?;

            if chat.last_message_id.is_none() && chat.chat_type == ChatType::Dm {
                self.event_bus
                    .publish(DomainEvent::ChatCreate { chat: chat.clone() });
            }

            self.event_bus.publish(DomainEvent::MessageCreated {
                chat: chat.clone(),
                message: message.clone(),
                member: initiator.clone(),
            });

            Ok(AddOrEditMessageCommandResult {
                message,
                attachments,
            })
        }
    }
}
