use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: i64,
    pub message_id: i64,
    pub chat_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,

    pub is_spoiler: bool,
    pub placeholder: Option<String>,
    pub duration_secs: Option<f32>,
    pub waveform: Option<String>,
}

pub struct CreateAttachmentRequest {
    pub id: i64,
    pub message_id: i64,
    pub chat_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub storage_key: String,
}

impl Attachment {
    pub fn new(request: CreateAttachmentRequest) -> Self {
        Self {
            id: request.id,
            message_id: request.message_id,
            chat_id: request.chat_id,
            filename: request.filename,
            content_type: request.content_type,
            size: request.size,
            storage_key: request.storage_key,
            created_at: Utc::now(),
            is_spoiler: false,
            placeholder: None,
            duration_secs: None,
            waveform: None,
        }
    }
}
