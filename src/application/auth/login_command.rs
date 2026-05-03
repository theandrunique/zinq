use std::sync::Arc;

use crate::{domain::auth::{User, data::user_repository::UserRepository}, error::Error, infra::id_generator::IdGenerator, state::AppState};


#[derive(Debug, validator::Validate, Clone)]
pub struct LoginCommand {
    pub username: String,
    pub password: String,
    pub totp: Option<String>,
}

pub struct LoginCommandHandler {
    id_gen: Arc<dyn IdGenerator>,
    user_repository: Arc<dyn UserRepository>,
}

impl LoginCommandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            id_gen: Arc::clone(&state.id_gen),
            user_repository: Arc::clone(&state.user_repository),
        }
    }

    pub async fn handle(&self, command: LoginCommand) -> Result<User, Error> {
        let user = self.user_repository
            .get_by_username(command.username)
            .await
            .map_err(|e| Error::InternalServerError(e))?;

        if user.is_none() {
            return Err(Error::AuthInvalidCredentials);
        }

        return Ok(user.unwrap());
    }
}
