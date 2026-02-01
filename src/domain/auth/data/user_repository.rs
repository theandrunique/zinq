use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::auth::User;

pub enum AddUserError {
    UsernameTaken,
    EmailTaken,
    InternalError(anyhow::Error),
}

pub enum UpdateUsernameError {
    UsernameTaken,
    InternalError(anyhow::Error),
}

pub enum UpdateEmailError {
    EmailTaken,
    InternalError(anyhow::Error),
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: User) -> Result<(), AddUserError>;

    async fn get_by_id(&self, user_id: i64) -> Result<Option<User>, anyhow::Error>;
    async fn get_by_email(&self, email: String) -> Result<Option<User>, anyhow::Error>;
    async fn get_by_username(&self, username: String) -> Result<Option<User>, anyhow::Error>;
    async fn get_by_ids(&self, user_ids: Vec<i64>) -> Result<Vec<User>, anyhow::Error>;

    async fn exists_by_email(&self, email: String) -> Result<bool, anyhow::Error>;
    async fn exists_by_username(&self, username: String) -> Result<bool, anyhow::Error>;

    async fn update_email(
        &self,
        user_id: i64,
        email: String,
        old_email: String,
        email_updated_timestamp: DateTime<Utc>,
        verified: bool,
    ) -> Result<(), UpdateEmailError>;

    async fn update_username(
        &self,
        user_id: i64,
        username: String,
        old_username: String,
        username_updated_timestamp: DateTime<Utc>,
    ) -> Result<(), UpdateUsernameError>;
}
