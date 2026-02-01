use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};

use super::schemas::common::UserPrivateSchema;
use crate::{
    application::{RegisterComandHandler, RegisterCommand},
    error::Error,
    state::AppState,
};

#[derive(Deserialize, Serialize)]
struct RegisterRequestSchema {
    pub username: String,
    pub email: String,
    pub global_name: String,
    pub password: String,
}

#[axum::debug_handler]
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequestSchema>,
) -> Result<Json<UserPrivateSchema>, Error> {
    let handler = RegisterComandHandler::new(&state);

    let command = RegisterCommand {
        username: payload.username,
        email: payload.email,
        global_name: payload.global_name,
        password: payload.password,
    };

    let user = handler.handle(command).await?;

    return Ok(Json(UserPrivateSchema::from(user)));
}

pub fn auth_router(state: AppState) -> Router {
    Router::new()
        .route("/sign-up", post(register))
        .with_state(state)
}
