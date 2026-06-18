use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        RequestHandler,
        auth::{LoginCommand, LoginCommandHandler},
    },
    error::Error,
    routers::auth_router::schemas::TokensResponseSchema,
    state::AppState,
};

#[derive(Deserialize, Serialize)]
pub struct LoginRequestSchema {
    pub username: String,
    pub password: String,
    pub totp: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequestSchema>,
) -> Result<Json<TokensResponseSchema>, Error> {
    let handler = LoginCommandHandler::new(&state);

    let command = LoginCommand {
        username: payload.username,
        password: payload.password,
        totp: payload.totp,
    };

    let result = handler.handle(command).await?;

    return Ok(Json(TokensResponseSchema {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        expires_in: result.expires_in,
    }));
}
