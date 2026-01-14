use async_trait::async_trait;

use crate::domain::auth::user_session::UserSession;

#[async_trait]
pub trait UserSessionRepository: Send + Sync {
    async fn add(session: UserSession);
    async fn get_by_id(user_id: i64, session_id: i64) -> Option<UserSession>;
    async fn get_sessions_by_user_id(user_id: i64) -> Vec<UserSession>;
    async fn update_token_id(session: UserSession);
    async fn remove_by_id(user_id: i64, session_id: i64);
}
