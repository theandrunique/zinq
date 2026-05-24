use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, Map, serde_as};

use crate::{
    application::{
        RequestHandler,
        chats::{
            CreateChatCommand, CreateChatCommandHandler, GetChatQuery, GetChatQueryHandler,
            GetUserChatsQuery, GetUserChatsQueryHandler,
        },
    },
    domain::chats::Chat,
    error::Error,
    infra::AuthUser,
    routers::schemas::common::ChatSchema,
    state::AppState,
};

#[axum::debug_handler]
pub async fn get_user_chats(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
) -> Result<Json<Vec<ChatSchema>>, Error> {
    let handler = GetUserChatsQueryHandler::new(&state);

    let command = GetUserChatsQuery {
        current_user_id: claims.sub,
    };

    let result = handler.handle(command).await?;

    let chats = result
        .into_iter()
        .map(|chat| ChatSchema::from(chat))
        .collect::<Vec<ChatSchema>>();

    return Ok(Json(chats));
}
