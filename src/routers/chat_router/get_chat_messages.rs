use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        messages::{GetMessagesQuery, GetMessagesQueryHandler},
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
pub struct GetMessagesQueryParams {
    before: Option<i64>,
    limit: Option<i32>,
}

#[axum::debug_handler]
pub async fn get_chat_messages(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdParam { chat_id }): Path<ChatIdParam>,
    Query(params): Query<GetMessagesQueryParams>,
) -> Result<Json<Vec<MessageSchema>>, Error> {
    let handler = GetMessagesQueryHandler::new(&state);

    let command = GetMessagesQuery {
        current_user_id: claims.sub,
        chat_id,
        before: params.before,
        limit: params.limit.unwrap_or(50),
    };

    let result = handler.handle(command).await?;

    Ok(Json(result.into()))
}
