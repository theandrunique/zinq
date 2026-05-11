use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, Map, serde_as};

use crate::{
    application::{
        RequestHandler,
        messages::{
            AddOrEditMessageCommand, AddOrEditMessageCommandHandler, MessageAttachmentInput,
        },
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::MessageSchema,
    state::AppState,
};

#[derive(Deserialize)]
pub struct ChatIdParam {
    chat_id: i64,
}

#[derive(Deserialize)]
pub struct MessageAttachmentDto {
    uploaded_filename: String,
    filename: String,
}

#[serde_as]
#[derive(Deserialize)]
pub struct CreateMessageRequest {
    content: String,
    #[serde_as(as = "Option<DisplayFromStr>")]
    referenced_message_id: Option<i64>,
    attachments: Option<Vec<MessageAttachmentDto>>,
}

#[axum::debug_handler]
pub async fn create_message(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdParam { chat_id }): Path<ChatIdParam>,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Json<MessageSchema>, Error> {
    let handler = AddOrEditMessageCommandHandler::new(&state);

    let command = AddOrEditMessageCommand {
        current_user_id: claims.sub,
        message_id: None,
        referenced_message_id: request.referenced_message_id,
        chat_id: chat_id,
        content: request.content,
        attachments: request
            .attachments
            .map(|attachments| {
                attachments
                    .into_iter()
                    .map(|attachment| MessageAttachmentInput {
                        uploaded_filename: attachment.uploaded_filename,
                        filename: attachment.filename,
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };

    let result = handler.handle(command).await?;

    return Ok(Json(MessageSchema::from(result)));
}
