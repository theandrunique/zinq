use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        RequestHandler,
        auth::{RefreshTokenCommand, RefreshTokenCommandHandler},
    },
    error::Error,
    routers::auth_router::schemas::TokensResponseSchema,
    state::AppState,
};

#[derive(Deserialize, Serialize)]
pub struct RefreshTokenRequestSchema {
    pub refresh_token: String,
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequestSchema>,
) -> Result<Json<TokensResponseSchema>, Error> {
    let handler = RefreshTokenCommandHandler::new(&state);

    let command = RefreshTokenCommand {
        refresh_token: payload.refresh_token,
    };

    let result = handler.handle(command).await?;

    Ok(Json(TokensResponseSchema {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        expires_in: result.expires_in,
    }))
}
