use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::error::Error;
use crate::infra::auth::jwt_handler::TokenClaims;
use crate::state::AppState;

pub struct AuthUser {
    pub claims: TokenClaims,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(Error::AuthInvalidToken)?;

        let token = if let Some(token) = auth_header.strip_prefix("Bearer ") {
            token
        } else if let Some(token) = auth_header.strip_prefix("bearer ") {
            token
        } else {
            return Err(Error::AuthInvalidToken);
        };

        let claims = state
            .jwt_handler
            .verify_access_token(token)
            .map_err(|err| {
                tracing::debug!("token verification failed: {err}");
                Error::AuthInvalidToken
            })?;

        Ok(AuthUser { claims })
    }
}
