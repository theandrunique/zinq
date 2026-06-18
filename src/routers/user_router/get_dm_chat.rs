use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    application::{
        RequestHandler,
        chats::{GetDMChatCommand, GetDMChatCommandHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::ChatSchema,
    state::AppState,
};

#[derive(Deserialize)]
pub struct UserIdParam {
    user_id: i64,
}

#[axum::debug_handler]
pub async fn get_dm_chat(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Path(UserIdParam { user_id }): Path<UserIdParam>,
) -> Result<Json<ChatSchema>, Error> {
    let handler = GetDMChatCommandHandler::new(&state);

    let command = GetDMChatCommand {
        current_user_id: claims.sub,
        user_id,
    };

    let result = handler.handle(command).await?;

    Ok(Json(result.into()))
}
