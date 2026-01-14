use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::auth::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn add(user: User) -> bool;

    async fn get_by_id(user_id: i64) -> Option<User>;
    async fn get_by_email(email: String) -> Option<User>;
    async fn get_by_username(username: String) -> Option<User>;
    async fn get_by_ids(user_ids: Vec<i64>) -> Vec<User>;

    async fn is_exists_by_email(email: String) -> bool;
    async fn is_exists_by_username(username: String) -> bool;

    async fn update_email(
        user_id: i64,
        email: String,
        old_email: String,
        email_updated_timestamp: DateTime<Utc>,
        is_email_verified: bool,
    ) -> bool;

    async fn update_username(
        user_id: i64,
        username: String,
        old_username: String,
        username_updated_timestamp: DateTime<Utc>,
    ) -> bool;

    async fn update_is_email_verified(user_id: i64, is_email_verified: bool);
    async fn update_totp_mfa_info(user: User);
    async fn update_avatar(user: User);
}
