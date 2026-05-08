use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, Map, serde_as};

use crate::{
    application::{
        RequestHandler,
        chats::{CreateChatCommand, CreateChatCommandHandler},
    },
    error::Error,
    infra::AuthUser,
    routers::schemas::common::ChatSchema,
    state::AppState,
};

#[serde_as]
#[derive(Deserialize, Serialize)]
struct CreateChatRequestSchema {
    pub name: String,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub members: Vec<i64>,
}

#[axum::debug_handler]
async fn create_chat(
    State(state): State<AppState>,
    AuthUser { claims }: AuthUser,
    Json(payload): Json<CreateChatRequestSchema>,
) -> Result<Json<ChatSchema>, Error> {
    let handler = CreateChatCommandHandler::new(&state);

    let command = CreateChatCommand {
        current_user_id: claims.sub.parse().unwrap(),
        name: payload.name,
        members: payload.members,
        permissions: None,
    };

    let result = handler.handle(command).await?;

    return Ok(Json(ChatSchema::from(result)));
}

pub fn chat_router(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_chat))
        .with_state(state)
}
