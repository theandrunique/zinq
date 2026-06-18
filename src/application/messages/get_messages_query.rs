use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::{
        attachments::{Attachment, data::AttachmentRepository},
        chats::data::{ChatLoadOptions, ChatLoader},
        messages::{Message, data::MessageRepository},
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

#[derive(Debug, Clone)]
pub struct GetMessagesQueryResult {
    pub messages: Vec<Message>,
    pub attachments: HashMap<i64, Vec<Attachment>>,
}

pub struct GetMessagesQueryHandler {
    chat_loader: Arc<dyn ChatLoader>,
    message_repository: Arc<dyn MessageRepository>,
    attachment_repository: Arc<dyn AttachmentRepository>,
}

impl GetMessagesQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_loader: Arc::clone(&state.chat_loader),
            message_repository: Arc::clone(&state.message_repository),
            attachment_repository: Arc::clone(&state.attachment_repository),
        }
    }
}

impl RequestHandler for GetMessagesQueryHandler {
    type Request = GetMessagesQuery;
    type Output = GetMessagesQueryResult;
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

        if messages.is_empty() {
            return Ok(GetMessagesQueryResult {
                messages,
                attachments: HashMap::new(),
            });
        }

        let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
        let attachments = self
            .attachment_repository
            .get_by_message_ids(request.chat_id, &message_ids)
            .await
            .map_err(|e| Error::InternalServerError(e))?;

        let mut attachments_by_message: HashMap<i64, Vec<Attachment>> = HashMap::new();
        for attachment in attachments {
            attachments_by_message
                .entry(attachment.message_id)
                .or_insert_with(Vec::new)
                .push(attachment);
        }

        Ok(GetMessagesQueryResult {
            messages,
            attachments: attachments_by_message,
        })
    }
}
