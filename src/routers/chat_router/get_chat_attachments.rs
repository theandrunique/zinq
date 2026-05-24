use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        chats::{GetAttachmentsQuery, GetAttachmentsQueryHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::AttachmentSchema,
    state::AppState,
};

#[derive(Deserialize)]
pub struct ChatIdParam {
    chat_id: i64,
}

#[derive(Deserialize)]
pub struct GetAttachmentsQueryParams {
    before: Option<i64>,
    limit: Option<i32>,
}

#[axum::debug_handler]
pub async fn get_chat_attachments(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdParam { chat_id }): Path<ChatIdParam>,
    Query(params): Query<GetAttachmentsQueryParams>,
) -> Result<Json<Vec<AttachmentSchema>>, Error> {
    let handler = GetAttachmentsQueryHandler::new(&state);

    let query = GetAttachmentsQuery {
        current_user_id: claims.sub,
        chat_id,
        before: params.before.unwrap_or(i64::MAX),
        limit: params.limit.unwrap_or(50),
    };

    let attachments = handler.handle(query).await?;

    Ok(Json(attachments.into_iter().map(|a| a.into()).collect()))
}
