use crate::domain::auth::data::user_repository::{
    AddUserError, UpdateEmailError, UpdateUsernameError, UserRepository,
};
use crate::infra::data::common::ScyllaCommon;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::DeserializeRow;
use scylla::client::session::Session;
use std::str::FromStr;
use std::sync::Arc;

use crate::domain::auth::{SessionLifetime, User};

#[derive(Debug, DeserializeRow)]
struct UserDb {
    user_id: i64,
    username: String,
    username_updated_timestamp: DateTime<Utc>,
    password_hash: String,
    password_updated_timestamp: DateTime<Utc>,
    avatar: Option<String>,
    sessions_lifetime: String,
    bio: Option<String>,
    global_name: String,
    is_active: bool,
    timestamp: DateTime<Utc>,
    totp_key: Option<Vec<u8>>,
    mfa: bool,
    email: String,
    is_email_verified: bool,
    email_updated_timestamp: DateTime<Utc>,
}

impl TryFrom<UserDb> for User {
    type Error = anyhow::Error;

    fn try_from(value: UserDb) -> Result<Self, Self::Error> {
        Ok(User {
            id: value.user_id,
            username: value.username,
            username_updated_timestamp: value.username_updated_timestamp,
            password_hash: value.password_hash,
            password_updated_timestamp: value.password_updated_timestamp,
            avatar: value.avatar,
            sessions_lifetime: SessionLifetime::from_str(&value.sessions_lifetime).map_err(
                |e| {
                    anyhow::anyhow!(
                        "Error parsing sessions_lifetime '{}': {}",
                        value.sessions_lifetime,
                        e
                    )
                },
            )?,
            bio: value.bio,
            global_name: value.global_name,
            is_active: value.is_active,
            timestamp: value.timestamp,
            totp_key: value.totp_key,
            mfa: value.mfa,
            email: value.email,
            is_email_verified: value.is_email_verified,
            email_updated_timestamp: value.email_updated_timestamp,
        })
    }
}

pub struct ScyllaUserRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaUserRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: session.clone(),
            common: ScyllaCommon::new(session),
        }
    }
}

impl ScyllaUserRepository {
    async fn insert_unique(
        &self,
        table: &str,
        key: &str,
        value: &str,
        user_id: i64,
    ) -> Result<bool, anyhow::Error> {
        let query = format!(
            "INSERT INTO {} ({}, user_id) VALUES (?, ?) IF NOT EXISTS",
            table, key
        );

        let row: Option<(bool, Option<String>, Option<i64>)> =
            self.common.exec_first(&query, (value, user_id)).await?;

        Ok(row.map(|r| r.0).unwrap_or(false))
    }

    async fn delete_index(
        &self,
        table: &str,
        field: &str,
        value: &str,
    ) -> Result<(), anyhow::Error> {
        let query = format!("DELETE FROM {} WHERE {} = ?", table, field);
        self.common.exec(&query, (value,)).await?;
        Ok(())
    }

    async fn insert_user(&self, u: &User) -> Result<(), anyhow::Error> {
        let query = r#"
            INSERT INTO users (
                user_id, username, username_updated_timestamp,
                password_hash, password_updated_timestamp,
                avatar, sessions_lifetime, bio, global_name,
                is_active, timestamp, totp_key, mfa,
                email, is_email_verified, email_updated_timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        self.common
            .exec(
                query,
                (
                    u.id,
                    u.username.clone(),
                    u.username_updated_timestamp,
                    u.password_hash.clone(),
                    u.password_updated_timestamp,
                    u.avatar.clone(),
                    u.sessions_lifetime.to_string(),
                    u.bio.clone(),
                    u.global_name.clone(),
                    u.is_active,
                    u.timestamp,
                    u.totp_key.clone(),
                    u.mfa,
                    u.email.clone(),
                    u.is_email_verified,
                    u.email_updated_timestamp,
                ),
            )
            .await?;

        Ok(())
    }
}

#[async_trait]
impl UserRepository for ScyllaUserRepository {
    async fn save(&self, user: User) -> Result<(), AddUserError> {
        if !self
            .insert_unique("users_by_username", "username", &user.username, user.id)
            .await
            .map_err(AddUserError::InternalError)?
        {
            return Err(AddUserError::UsernameTaken);
        }

        if !self
            .insert_unique("users_by_email", "email", &user.email, user.id)
            .await
            .map_err(AddUserError::InternalError)?
        {
            self.delete_index("users_by_username", "username", &user.username)
                .await
                .map_err(AddUserError::InternalError)?;
            return Err(AddUserError::EmailTaken);
        }

        if let Err(e) = self.insert_user(&user).await {
            let _ = self
                .delete_index("users_by_username", "username", &user.username)
                .await;
            let _ = self
                .delete_index("users_by_email", "email", &user.email)
                .await;
            return Err(AddUserError::InternalError(e));
        }

        Ok(())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<User>, anyhow::Error> {
        let row: Option<UserDb> = self
            .common
            .exec_first("SELECT * FROM users WHERE user_id = ?", (id,))
            .await?;

        row.map(User::try_from).transpose()
    }

    async fn get_by_email(&self, email: String) -> Result<Option<User>, anyhow::Error> {
        let row: Option<(i64,)> = self
            .common
            .exec_first(
                "SELECT user_id FROM users_by_email WHERE email = ?",
                (email,),
            )
            .await?;

        match row {
            Some((id,)) => self.get_by_id(id).await,
            None => Ok(None),
        }
    }

    async fn get_by_username(&self, username: String) -> Result<Option<User>, anyhow::Error> {
        let row: Option<(i64,)> = self
            .common
            .exec_first(
                "SELECT user_id FROM users_by_username WHERE username = ?",
                (username,),
            )
            .await?;

        match row {
            Some((id,)) => self.get_by_id(id).await,
            None => Ok(None),
        }
    }

    async fn get_by_ids(&self, user_ids: Vec<i64>) -> Result<Vec<User>, anyhow::Error> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let user_dbs: Vec<UserDb> = self
            .common
            .exec_all("SELECT * FROM users WHERE user_id IN ?", (user_ids,))
            .await?;

        user_dbs.into_iter().map(User::try_from).collect()
    }

    async fn exists_by_email(&self, email: String) -> Result<bool, anyhow::Error> {
        let result: Option<(i64,)> = self
            .common
            .exec_first(
                "SELECT COUNT(1) FROM users_by_email WHERE email = ?",
                (email,),
            )
            .await?;
        Ok(result.is_some())
    }

    async fn exists_by_username(&self, username: String) -> Result<bool, anyhow::Error> {
        let result: Option<(i64,)> = self
            .common
            .exec_first(
                "SELECT COUNT(1) FROM users_by_username WHERE username = ?",
                (username,),
            )
            .await?;
        Ok(result.is_some())
    }

    async fn update_email(
        &self,
        user_id: i64,
        email: String,
        old_email: String,
        ts: DateTime<Utc>,
        verified: bool,
    ) -> Result<(), UpdateEmailError> {
        if !self
            .insert_unique("users_by_email", "email", &email, user_id)
            .await
            .map_err(UpdateEmailError::InternalError)?
        {
            return Err(UpdateEmailError::EmailTaken);
        }

        let query = "UPDATE users SET email = ?, email_updated_timestamp = ?, is_email_verified = ? WHERE user_id = ?";

        self.common
            .exec(query, (email.clone(), ts, verified, user_id))
            .await
            .map_err(UpdateEmailError::InternalError)?;
        self.delete_index("users_by_email", "email", &old_email)
            .await
            .map_err(UpdateEmailError::InternalError)?;

        Ok(())
    }

    async fn update_username(
        &self,
        user_id: i64,
        username: String,
        old_username: String,
        ts: DateTime<Utc>,
    ) -> Result<(), UpdateUsernameError> {
        if !self
            .insert_unique("users_by_username", "username", &username, user_id)
            .await
            .map_err(UpdateUsernameError::InternalError)?
        {
            return Err(UpdateUsernameError::UsernameTaken);
        }

        let query =
            "UPDATE users SET username = ?, username_updated_timestamp = ? WHERE user_id = ?";

        self.common
            .exec(query, (username.clone(), ts, user_id))
            .await
            .map_err(UpdateUsernameError::InternalError)?;
        self.delete_index("users_by_username", "username", &old_username)
            .await
            .map_err(UpdateUsernameError::InternalError)?;

        Ok(())
    }
}
