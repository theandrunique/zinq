use async_trait::async_trait;

use crate::domain::event_log::EventLog;

#[async_trait]
pub trait EventLogRepository: Send + Sync {
    async fn save(&self, event: EventLog) -> Result<(), anyhow::Error>;

    async fn get_event_logs(
        &self,
        user_id: i64,
        after_event_id: i64,
        limit: i32,
    ) -> Result<Vec<EventLog>, anyhow::Error>;
}
