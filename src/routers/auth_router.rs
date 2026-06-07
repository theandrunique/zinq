use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};

use super::schemas::common::UserPrivateSchema;
use crate::{
    application::{
        RequestHandler,
        auth::{
            LoginCommand, LoginCommandHandler, RefreshTokenCommand, RefreshTokenCommandHandler,
            RegisterComandHandler, RegisterCommand,
        },
    },
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

#[derive(Deserialize, Serialize)]
struct LoginRequestSchema {
    pub username: String,
    pub password: String,
    pub totp: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct LoginResponseSchema {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequestSchema>,
) -> Result<Json<LoginResponseSchema>, Error> {
    let handler = LoginCommandHandler::new(&state);

    let command = LoginCommand {
        username: payload.username,
        password: payload.password,
        totp: payload.totp,
    };

    let result = handler.handle(command).await?;

    return Ok(Json(LoginResponseSchema {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        expires_in: result.expires_in,
    }));
}

#[derive(Deserialize, Serialize)]
struct RefreshTokenRequestSchema {
    pub refresh_token: String,
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequestSchema>,
) -> Result<Json<LoginResponseSchema>, Error> {
    let handler = RefreshTokenCommandHandler::new(&state);

    let command = RefreshTokenCommand {
        refresh_token: payload.refresh_token,
    };

    let result = handler.handle(command).await?;

    return Ok(Json(LoginResponseSchema {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        expires_in: result.expires_in,
    }));
}

pub fn auth_router(state: AppState) -> Router {
    Router::new()
        .route("/sign-up", post(register))
        .route("/sign-in", post(login))
        .route("/refresh", post(refresh_token))
        .with_state(state)
}
