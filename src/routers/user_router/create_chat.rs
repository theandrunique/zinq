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
        chats::{CreateChatCommand, CreateChatCommandHandler, GetChatQuery, GetChatQueryHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::ChatSchema,
    state::AppState,
};

#[serde_as]
#[derive(Deserialize)]
pub struct CreateChatRequestSchema {
    pub name: String,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub members: Vec<i64>,
}

#[axum::debug_handler]
pub async fn create_chat(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Json(payload): Json<CreateChatRequestSchema>,
) -> Result<Json<ChatSchema>, Error> {
    let handler = CreateChatCommandHandler::new(&state);

    let command = CreateChatCommand {
        current_user_id: claims.sub,
        name: payload.name,
        members: payload.members,
        permissions: None,
    };

    let result = handler.handle(command).await?;

    Ok(Json(ChatSchema::from(result)))
}
