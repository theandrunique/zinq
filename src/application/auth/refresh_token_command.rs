use std::sync::Arc;

use crate::{
    application::{RequestHandler, auth::login_command::LoginCommandResult},
    domain::auth::data::user_session_repository::UserSessionRepository,
    error::Error,
    infra::JwtHandler,
    state::AppState,
};

pub struct RefreshTokenCommand {
    pub refresh_token: String,
}

pub struct RefreshTokenCommandHandler {
    user_session_repository: Arc<dyn UserSessionRepository>,
    jwt_handler: Arc<dyn JwtHandler>,
}

impl RefreshTokenCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            user_session_repository: Arc::clone(&state.user_session_repository),
            jwt_handler: Arc::clone(&state.jwt_handler),
        }
    }
}

impl RequestHandler for RefreshTokenCommandHandler {
    type Request = RefreshTokenCommand;
    type Output = LoginCommandResult;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let claims = self
            .jwt_handler
            .verify_refresh_token(&request.refresh_token)
            .map_err(|_| Error::AuthInvalidToken)?;

        let sessions = self
            .user_session_repository
            .get_sessions_by_user_id(claims.sub)
            .await
            .map_err(|e| Error::InternalServerError(e))?;

        let session = sessions
            .into_iter()
            .find(|s| s.token_id == claims.session_id)
            .ok_or(Error::AuthInvalidToken)?;

        let access_token = self
            .jwt_handler
            .generate_access_token(session.user_id, session.token_id)
            .map_err(|e| Error::InternalServerError(e))?;

        let refresh_token = self
            .jwt_handler
            .generate_refresh_token(session.user_id, session.token_id, 604800)
            .map_err(|e| Error::InternalServerError(e))?;

        Ok(LoginCommandResult {
            access_token,
            refresh_token,
            expires_in: 604800,
        })
    }
}
