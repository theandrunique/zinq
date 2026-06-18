use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::auth::{
        UserSession, UserSessionCreateRequest,
        data::{user_repository::UserRepository, user_session_repository::UserSessionRepository},
    },
    error::Error,
    infra::{
        auth::{hash_handler::HashHandler, jwt_handler::JwtHandler},
        id_generator::IdGenerator,
    },
    state::AppState,
};

#[derive(Debug, validator::Validate, Clone)]
pub struct LoginCommand {
    pub username: String,
    pub password: String,
    pub totp: Option<String>,
}

pub struct LoginCommandResult {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

pub struct LoginCommandHandler {
    id_gen: Arc<dyn IdGenerator>,
    user_repository: Arc<dyn UserRepository>,
    session_repository: Arc<dyn UserSessionRepository>,
    jwt_handler: Arc<dyn JwtHandler>,
    hash_handler: Arc<dyn HashHandler>,
}

impl LoginCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            id_gen: Arc::clone(&state.id_gen),
            user_repository: Arc::clone(&state.user_repository),
            session_repository: Arc::clone(&state.user_session_repository),
            jwt_handler: Arc::clone(&state.jwt_handler),
            hash_handler: Arc::clone(&state.hash_handler),
        }
    }
}

impl RequestHandler for LoginCommandHandler {
    type Request = LoginCommand;
    type Output = LoginCommandResult;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let user = self
            .user_repository
            .get_by_username(&request.username)
            .await
            .map_err(Error::InternalServerError)?
            .ok_or(Error::AuthInvalidCredentials)?;

        if !self
            .hash_handler
            .verify(&request.password, &user.password_hash)
            .await?
        {
            return Err(Error::AuthInvalidCredentials);
        }

        let session = UserSession::create(UserSessionCreateRequest {
            id: self.id_gen.gen_id().await,
            user_id: user.id,
            device_name: "".to_string(),
            client_name: "".to_string(),
            location: "".to_string(),
        });

        self.session_repository.save(&session).await?;

        let access_token = self
            .jwt_handler
            .generate_access_token(user.id, session.token_id)?;
        let refresh_token =
            self.jwt_handler
                .generate_refresh_token(user.id, session.token_id, 604800)?;

        Ok(LoginCommandResult {
            access_token,
            refresh_token,
            expires_in: 604800,
        })
    }
}
