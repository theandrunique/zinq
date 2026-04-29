use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, client::session::Session};

use crate::{
    domain::attachments::{Attachment, data::AttachmentRepository},
    infra::data::common::ScyllaCommon,
};

#[derive(Debug, DeserializeRow)]
struct AttachmentDb {
    chat_id: i64,
    attachment_id: i64,
    message_id: i64,
    content_type: String,
    duration_secs: Option<f32>,
    filename: String,
    is_spoiler: bool,
    placeholder: Option<String>,
    signed_url: String,
    signed_url_expires: DateTime<Utc>,
    size: i64,
    waveform: Option<String>,
    timestamp: DateTime<Utc>,
}

impl TryFrom<AttachmentDb> for Attachment {
    type Error = anyhow::Error;

    fn try_from(value: AttachmentDb) -> Result<Self, Self::Error> {
        Ok(Attachment {
            id: value.attachment_id,
            message_id: value.message_id,
            chat_id: value.chat_id,
            filename: value.filename,
            content_type: value.content_type,
            size: value.size,
            signed_url: value.signed_url,
            signed_url_expires: value.signed_url_expires,
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
        let query = "
            SELECT *
            FROM attachments_by_id
            WHERE chat_id = ? AND attachment_id = ?
            LIMIT 1
        ";

        let row: Option<AttachmentDb> = self
            .common
            .exec_first(query, (chat_id, attachment_id))
            .await?;
        row.map(Attachment::try_from).transpose()
    }

    async fn get_channel_attachments(
        &self,
        chat_id: i64,
        before_message_id: i64,
        limit: i32,
    ) -> Result<Vec<Attachment>, anyhow::Error> {
        let query = "
            SELECT *
            FROM attachments_by_message_id
            WHERE chat_id = ? AND message_id < ?
            LIMIT ?
        ";

        let rows: Vec<AttachmentDb> = self
            .common
            .exec_all(query, (chat_id, before_message_id, limit))
            .await?;

        rows.into_iter().map(Attachment::try_from).collect()
    }

    async fn update_signed_urls(&self, attachments: Vec<Attachment>) -> Result<(), anyhow::Error> {
        let query = "
            UPDATE attachments_by_message_id
            SET signed_url = ?, signed_url_expires = ?
            WHERE chat_id = ? AND message_id = ? AND attachment_id = ?
        ";

        for attachment in attachments {
            self.common
                .exec(
                    query,
                    (
                        attachment.signed_url.clone(),
                        attachment.signed_url_expires,
                        attachment.chat_id,
                        attachment.message_id,
                        attachment.id,
                    ),
                )
                .await?;
        }

        Ok(())
    }
}
