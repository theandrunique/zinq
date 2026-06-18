use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        RequestHandler,
        auth::{RegisterComandHandler, RegisterCommand},
    },
    error::Error,
    routers::schemas::common::UserPrivateSchema,
    state::AppState,
};

#[derive(Deserialize, Serialize)]
pub struct RegisterRequestSchema {
    pub username: String,
    pub email: String,
    pub global_name: String,
    pub password: String,
}

#[axum::debug_handler]
pub async fn register(
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

    Ok(Json(UserPrivateSchema::from(user)))
}
