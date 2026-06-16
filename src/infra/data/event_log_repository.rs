use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, SerializeRow, client::session::Session};

use crate::{
    domain::event_log::{EventLog, EventLogType, data::EventLogRepository},
    infra::data::common::ScyllaCommon,
};

#[derive(SerializeRow, DeserializeRow)]
struct EventLogDb {
    user_id: i64,
    event_id: i64,
    event_type: String,
    timestamp: DateTime<Utc>,
}

impl TryFrom<EventLogDb> for EventLog {
    type Error = anyhow::Error;

    fn try_from(value: EventLogDb) -> Result<Self, Self::Error> {
        let event_type: EventLogType = serde_json::from_str(&value.event_type)?;
        Ok(EventLog {
            user_id: value.user_id,
            event_id: value.event_id,
            event_type,
            created_at: value.timestamp,
        })
    }
}

pub struct ScyllaEventLogRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaEventLogRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: session.clone(),
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl EventLogRepository for ScyllaEventLogRepository {
    async fn save(&self, event: &EventLog) -> Result<(), anyhow::Error> {
        let query = "INSERT INTO user_event_log (user_id, event_id, event_type, timestamp) VALUES (?, ?, ?, ?)";

        let event_type = serde_json::to_string(&event.event_type)?;

        self.common
            .exec(
                query,
                (event.user_id, event.event_id, event_type, event.created_at),
            )
            .await?;

        Ok(())
    }

    async fn get_event_logs(
        &self,
        user_id: i64,
        after_event_id: i64,
        limit: i32,
    ) -> Result<Vec<EventLog>, anyhow::Error> {
        let query = "SELECT user_id, event_id, event_type, timestamp FROM user_event_log WHERE user_id = ? AND event_id > ? LIMIT ?";

        let rows: Vec<EventLogDb> = self
            .common
            .exec_all(query, (user_id, after_event_id, limit))
            .await?;

        rows.into_iter()
            .map(EventLog::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
