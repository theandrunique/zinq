use std::sync::Arc;

use crate::{
    domain::auth::data::user_repository::UserRepository,
    error::Error,
    infra::{hash_handler::HashHandler, id_generator::IdGenerator, jwt_handler::JwtHandler},
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
    jwt_handler: Arc<dyn JwtHandler>,
    hash_handler: Arc<dyn HashHandler>,
}

impl LoginCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            id_gen: Arc::clone(&state.id_gen),
            user_repository: Arc::clone(&state.user_repository),
            jwt_handler: Arc::clone(&state.jwt_handler),
            hash_handler: Arc::clone(&state.hash_handler),
        }
    }

    pub async fn handle(&self, command: LoginCommand) -> Result<LoginCommandResult, Error> {
        let user = self
            .user_repository
            .get_by_username(command.username)
            .await
            .map_err(|e| Error::InternalServerError(e))?
            .ok_or(Error::AuthInvalidCredentials)?;

        if !self.hash_handler.verify(&command.password, &user.password_hash).await? {
            return Err(Error::AuthInvalidCredentials);
        }

        let access_token = self
            .jwt_handler
            .generate_access_token(&user.id.to_string(), "kek")
            .await?;
        let refresh_token = self
            .jwt_handler
            .generate_refresh_token(&user.id.to_string(), "kek", 123)
            .await?;

        return Ok(LoginCommandResult {
            access_token: access_token,
            expires_in: 123,
            refresh_token: refresh_token,
        });
    }
}
