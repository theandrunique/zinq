use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        chats::{GetChatQuery, GetChatQueryHandler},
        messages::{
            CreateCloudAttachmentsCommand, CreateCloudAttachmentsCommandHandler,
            CreateCloudAttachmentsResponse, UploadAttachmentDto,
        },
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::ChatSchema,
    state::AppState,
};

#[derive(Deserialize)]
pub struct AttachmentInput {
    id: Option<String>,
    filename: String,
    filesize: i64,
}

#[derive(Deserialize)]
pub struct CreateCloudAttachmentRequestSchema {
    files: Vec<AttachmentInput>,
}

#[derive(Deserialize)]
pub struct ChatIdParam {
    chat_id: i64,
}

#[axum::debug_handler]
pub async fn create_cloud_attachment(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdParam { chat_id }): Path<ChatIdParam>,
    Json(request): Json<CreateCloudAttachmentRequestSchema>,
) -> Result<Json<CreateCloudAttachmentsResponse>, Error> {
    let handler = CreateCloudAttachmentsCommandHandler::new(&state);

    let command = CreateCloudAttachmentsCommand {
        current_user_id: claims.sub,
        chat_id,
        files: request
            .files
            .into_iter()
            .map(|attachment| UploadAttachmentDto {
                id: attachment.id,
                filename: attachment.filename,
                filesize: attachment.filesize,
            })
            .collect(),
    };

    let result = handler.handle(command).await?;

    Ok(Json(result))
}
