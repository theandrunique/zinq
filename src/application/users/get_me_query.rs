use std::sync::Arc;

use crate::{
    application::RequestHandler,
    domain::auth::{User, data::user_repository::UserRepository},
    error::Error,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct GetMeQuery {
    pub current_user_id: i64,
}

pub struct GetMeQueryHandler {
    user_repository: Arc<dyn UserRepository>,
}

impl GetMeQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            user_repository: Arc::clone(&state.user_repository),
        }
    }
}

impl RequestHandler for GetMeQueryHandler {
    type Request = GetMeQuery;
    type Output = User;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let user = self
            .user_repository
            .get_by_id(request.current_user_id)
            .await
            .map_err(|e| Error::InternalServerError(e))?
            .ok_or(Error::UserNotFound(request.current_user_id))?;

        Ok(user)
    }
}
