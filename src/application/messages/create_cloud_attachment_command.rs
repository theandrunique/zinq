use std::sync::Arc;

use serde::Serialize;

use crate::{
    application::RequestHandler,
    core::ValidateExt,
    domain::chats::{ChatPermissions, data::ChatLoadOptions, data::ChatLoader},
    error::Error,
    state::AppState,
};

const MAX_FILE_SIZE: i64 = 10 * 1024 * 1024;

#[derive(Debug, validator::Validate, Clone)]
pub struct UploadAttachmentDto {
    pub id: Option<String>,
    pub filename: String,
    pub filesize: i64,
}

#[derive(Debug, validator::Validate, Clone)]
pub struct CreateCloudAttachmentsCommand {
    pub current_user_id: i64,
    pub chat_id: i64,
    pub files: Vec<UploadAttachmentDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudAttachmentSuccess {
    pub id: Option<String>,
    pub upload_filename: String,
    pub upload_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudAttachmentError {
    pub id: Option<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCloudAttachmentsResponse {
    pub results: Vec<CloudAttachmentSuccess>,
    pub errors: Vec<CloudAttachmentError>,
}

pub struct CreateCloudAttachmentsCommandHandler {
    chat_loader: Arc<dyn ChatLoader>,
    attachment_service: Arc<crate::application::services::AttachmentService>,
}

impl CreateCloudAttachmentsCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            chat_loader: Arc::clone(&state.chat_loader),
            attachment_service: Arc::clone(&state.attachment_service),
        }
    }
}

impl RequestHandler for CreateCloudAttachmentsCommandHandler {
    type Request = CreateCloudAttachmentsCommand;
    type Output = CreateCloudAttachmentsResponse;
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
            .ok_or_else(|| Error::ChatNotFound(request.chat_id))?;

        if !chat.has_member(request.current_user_id) {
            return Err(Error::UserNotMember {
                user_id: request.current_user_id,
                chat_id: request.chat_id,
            });
        }

        if !chat.has_permission(request.current_user_id, ChatPermissions::SEND_FILES) {
            return Err(Error::InsufficientPermissions {
                permission: ChatPermissions::SEND_FILES,
                chat_id: request.chat_id,
            });
        }

        let mut results = Vec::new();
        let mut errors = Vec::new();

        for file in request.files {
            if file.filesize <= 0 || file.filesize > MAX_FILE_SIZE {
                errors.push(CloudAttachmentError {
                    id: file.id,
                    errors: vec![format!(
                        "File size must be between 1 and {} bytes",
                        MAX_FILE_SIZE
                    )],
                });
                continue;
            }

            let upload_url = self
                .attachment_service
                .generate_upload_url(file.filesize, request.chat_id, &file.filename)
                .await
                .map_err(Error::InternalServerError)?;

            results.push(CloudAttachmentSuccess {
                id: file.id,
                upload_filename: upload_url.storage_key,
                upload_url: upload_url.upload_url,
            });
        }

        Ok(CreateCloudAttachmentsResponse { results, errors })
    }
}
