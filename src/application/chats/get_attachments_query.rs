use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::{
        attachments::{Attachment, data::AttachmentRepository},
        chats::data::{ChatLoadOptions, ChatLoader},
    },
    error::Error,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct GetAttachmentsQuery {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub before: i64,
    pub limit: i32,
}

pub struct GetAttachmentsQueryHandler {
    chat_loader: Arc<dyn ChatLoader>,
    attachment_repository: Arc<dyn AttachmentRepository>,
}

impl GetAttachmentsQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_loader: Arc::clone(&state.chat_loader),
            attachment_repository: Arc::clone(&state.attachment_repository),
        }
    }
}

impl RequestHandler for GetAttachmentsQueryHandler {
    type Request = GetAttachmentsQuery;
    type Output = Vec<Attachment>;
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

        let attachments = self
            .attachment_repository
            .get_channel_attachments(request.chat_id, request.before, request.limit)
            .await
            .map_err(|e| Error::InternalServerError(e))?;

        Ok(attachments)
    }
}
