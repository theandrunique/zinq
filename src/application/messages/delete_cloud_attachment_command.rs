use std::sync::Arc;

use crate::{
    application::RequestHandler, domain::attachments::data::AttachmentRepository, error::Error,
    state::AppState,
};

pub struct DeleteCloudAttachmentCommand {
    pub upload_filename: String,
}

pub struct DeleteCloudAttachmentCommandHandler {
    attachment_repository: Arc<dyn AttachmentRepository>,
    attachment_service: Arc<crate::application::services::AttachmentService>,
}

impl DeleteCloudAttachmentCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            attachment_repository: Arc::clone(&state.attachment_repository),
            attachment_service: Arc::clone(&state.attachment_service),
        }
    }
}

impl RequestHandler for DeleteCloudAttachmentCommandHandler {
    type Request = DeleteCloudAttachmentCommand;
    type Output = ();
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let parsed = self
            .attachment_service
            .parse_storage_key(&request.upload_filename)
            .ok_or_else(|| Error::AttachmentInvalidUploadFilename {
                upload_filename: request.upload_filename.clone(),
            })?;

        let exists = self
            .attachment_service
            .is_object_exists(&request.upload_filename)
            .await
            .map_err(Error::InternalServerError)?;

        if !exists {
            return Err(Error::AttachmentObjectNotFound {
                upload_filename: request.upload_filename,
            });
        }

        let attachment = self
            .attachment_repository
            .get_by_id(parsed.chat_id, parsed.attachment_id)
            .await
            .map_err(Error::InternalServerError)?;

        if let Some(att) = attachment {
            return Err(Error::AttachmentInUse {
                upload_filename: request.upload_filename,
                attachment_id: att.id,
            });
        }

        self.attachment_service
            .delete_object(&request.upload_filename)
            .await
            .map_err(Error::InternalServerError)?;

        Ok(())
    }
}
