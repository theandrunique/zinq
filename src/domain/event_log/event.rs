use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::event_log::EventLogType;

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub event_id: i64,
    pub event_type: EventLogType,
    pub created_at: DateTime<Utc>,
    pub recipients: Vec<i64>,
}
