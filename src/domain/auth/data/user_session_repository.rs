use async_trait::async_trait;

use crate::domain::auth::user_session::UserSession;

#[async_trait]
pub trait UserSessionRepository: Send + Sync {
    async fn save(&self, session: UserSession) -> Result<(), anyhow::Error>;
    async fn get_by_id(
        &self,
        user_id: i64,
        session_id: i64,
    ) -> Result<Option<UserSession>, anyhow::Error>;
    async fn get_sessions_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserSession>, anyhow::Error>;
    async fn update_token_id(&self, session: UserSession) -> Result<(), anyhow::Error>;
    async fn remove_by_id(&self, user_id: i64, session_id: i64) -> Result<(), anyhow::Error>;
}
