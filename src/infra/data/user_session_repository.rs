use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, client::session::Session};

use crate::{
    domain::auth::{UserSession, data::user_session_repository::UserSessionRepository},
    infra::data::common::ScyllaCommon,
};

#[derive(Debug, DeserializeRow)]
struct UserSessionDb {
    user_id: i64,
    session_id: i64,
    client_name: String,
    timestamp: DateTime<Utc>,
    device_name: String,
    last_used_timestamp: DateTime<Utc>,
    location: String,
    token_id: uuid::Uuid,
}

impl TryFrom<UserSessionDb> for UserSession {
    type Error = anyhow::Error;

    fn try_from(value: UserSessionDb) -> Result<Self, Self::Error> {
        Ok(UserSession {
            id: value.user_id,
            user_id: value.user_id,
            token_id: value.token_id,
            device_name: value.device_name,
            client_name: value.client_name,
            location: value.location,
            last_refresh_at: value.last_used_timestamp,
            created_at: value.timestamp,
        })
    }
}

pub struct ScyllaUserSessionRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaUserSessionRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: session.clone(),
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl UserSessionRepository for ScyllaUserSessionRepository {
    async fn save(&self, session: &UserSession) -> Result<(), anyhow::Error> {
        let query = "
            INSERT INTO sessions (
                user_id,
                session_id,
                client_name,
                timestamp,
                device_name,
                last_used_timestamp,
                location,
                token_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        self.common
            .exec(
                query,
                (
                    session.user_id,
                    session.id,
                    &session.client_name,
                    session.created_at,
                    &session.device_name,
                    session.last_refresh_at,
                    &session.location,
                    session.token_id,
                ),
            )
            .await?;
        Ok(())
    }

    async fn get_by_id(
        &self,
        user_id: i64,
        session_id: i64,
    ) -> Result<Option<UserSession>, anyhow::Error> {
        let row: Option<UserSessionDb> = self
            .common
            .exec_first(
                "SELECT * FROM sessions WHERE user_id = ? AND session_id = ?",
                (user_id, session_id),
            )
            .await?;

        row.map(UserSession::try_from).transpose()
    }

    async fn get_user_sessions(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserSession>, anyhow::Error> {
        let user_dbs: Vec<UserSessionDb> = self
            .common
            .exec_all("SELECT * FROM sessions WHERE user_id = ?", (user_id,))
            .await?;

        user_dbs.into_iter().map(UserSession::try_from).collect()
    }

    async fn update_token_id(&self, session: &UserSession) -> Result<(), anyhow::Error> {
        self.common
            .exec(
                "UPDATE sessions SET last_used_timestamp = ?, token_id = ? WHERE user_id = ? AND session_id = ?",
                (session.last_refresh_at, session.token_id, session.user_id, session.id),
            )
            .await?;
        Ok(())
    }

    async fn remove_by_id(&self, user_id: i64, session_id: i64) -> Result<(), anyhow::Error> {
        self.common
            .exec(
                "DELETE FROM sessions WHERE user_id = ? AND session_id = ?",
                (user_id, session_id),
            )
            .await?;
        Ok(())
    }
}
