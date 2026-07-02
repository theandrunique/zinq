use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        chats::{GetChatQuery, GetChatQueryHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::ChatSchema,
    state::AppState,
};

#[derive(Deserialize)]
pub struct ChatIdParam {
    chat_id: i64,
}

#[axum::debug_handler]
pub async fn get_chat(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(ChatIdParam { chat_id }): Path<ChatIdParam>,
) -> Result<Json<ChatSchema>, Error> {
    let handler = GetChatQueryHandler::new(&state);

    let command = GetChatQuery {
        current_user_id: claims.sub,
        chat_id,
    };

    let result = handler.handle(command).await?;

    Ok(Json(ChatSchema::from_chat_for_user(result, claims.sub)))
}
