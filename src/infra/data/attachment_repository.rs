use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::client::session::Session;

use crate::{
    domain::messages::{attachment::Attachment, data::AttachmentRepository},
    infra::data::common::ScyllaCommon,
};

struct AttachmentDb {
    chat_id: i64,
    attachment_id: i64,
    message_id: i64,
    content_type: String,
    duration_secs: Option<f32>,
    filename: String,
    is_spoiler: bool,
    placeholder: Option<String>,
    presigned_url: String,
    presigned_url_expired_timestamp: DateTime<Utc>,
    size: i64,
    waveform: Option<String>,
    timestamp: DateTime<Utc>,
}

impl TryFrom<AttachmentDb> for Attachment {
    type Error = anyhow::Error;

    fn try_from(value: AttachmentDb) -> Result<Self, Self::Error> {
        Ok(Attachment {
            id: value.attachment_id,
            message_id: Some(value.message_id),
            chat_id: value.chat_id,
            filename: value.filename,
            content_type: value.content_type,
            size: value.size,
            pre_signed_url: value.presigned_url,
            pre_signed_url_expires_timestamp: value.presigned_url_expired_timestamp,
            placeholder: value.placeholder,
            duration_secs: value.duration_secs,
            waveform: value.waveform,
            is_spoiler: value.is_spoiler,
            timestamp: value.timestamp,
        })
    }
}

pub struct ScyllaAttachmentRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaAttachmentRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: session.clone(),
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl AttachmentRepository for ScyllaAttachmentRepository {
    async fn get_by_id(
        &self,
        chat_id: i64,
        attachment_id: i64,
    ) -> Result<Option<Attachment>, anyhow::Error> {
        todo!()
    }

    async fn get_channel_attachments(
        &self,
        chat_id: i64,
        before_message_id: i64,
        limit: i32,
    ) -> Result<Vec<Attachment>, anyhow::Error> {
        todo!()
    }

    async fn update_pre_signed_urls(
        &self,
        chat_id: i64,
        attachments: Vec<Attachment>,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
}
