use chrono::{DateTime, Utc};

pub struct Attachment {
    pub id: i64,
    pub message_id: Option<i64>,
    pub channel_id: i64,
    pub filename: i64,
    pub content_type: i64,
    pub size: i64,
    pub pre_signed_url: String,
    pub pre_signed_url_expires_timestamp: DateTime<Utc>,
    pub placeholder: Option<String>,
    pub duration_secs: Option<f32>,
    pub waveform: Option<String>,
    pub is_spoiler: bool,
    pub timestamp: DateTime<Utc>,
}
