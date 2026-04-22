use chrono::{DateTime, Utc};

pub struct Attachment {
    pub id: i64,
    pub message_id: i64,
    pub chat_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub signed_url: String,
    pub signed_url_expires: DateTime<Utc>,
    pub is_spoiler: bool,
    pub placeholder: Option<String>,
    pub duration_secs: Option<f32>,
    pub waveform: Option<String>,
    pub timestamp: DateTime<Utc>,
}
