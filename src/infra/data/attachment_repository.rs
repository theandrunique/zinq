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
    storage_key: Option<String>,
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
            storage_key: value.storage_key.unwrap_or_default(),
            placeholder: value.placeholder,
            duration_secs: value.duration_secs,
            waveform: value.waveform,
            is_spoiler: value.is_spoiler,
            created_at: value.timestamp,
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
    async fn save(&self, attachment: Attachment) -> Result<(), anyhow::Error> {
        let query = "
            INSERT INTO attachments_by_message_id (
                chat_id,
                attachment_id,
                message_id,
                content_type,
                duration_secs,
                filename,
                is_spoiler,
                placeholder,
                storage_key,
                size,
                waveform,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ";

        self.common
            .exec(
                query,
                (
                    attachment.chat_id,
                    attachment.id,
                    attachment.message_id,
                    attachment.content_type,
                    attachment.duration_secs,
                    attachment.filename,
                    attachment.is_spoiler,
                    attachment.placeholder,
                    attachment.storage_key,
                    attachment.size,
                    attachment.waveform,
                    attachment.created_at,
                ),
            )
            .await?;

        Ok(())
    }

    async fn bulk_save(&self, attachments: &[Attachment]) -> Result<(), anyhow::Error> {
        for attachment in attachments {
            self.save(attachment.clone()).await?;
        }
        Ok(())
    }

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

    async fn get_by_message_ids(
        &self,
        chat_id: i64,
        message_ids: &[i64],
    ) -> Result<Vec<Attachment>, anyhow::Error> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = "
            SELECT *
            FROM attachments_by_message_id
            WHERE chat_id = ? AND message_id IN ?
        ";

        let rows: Vec<AttachmentDb> = self
            .common
            .exec_all(query, (chat_id, message_ids))
            .await?;

        rows.into_iter().map(Attachment::try_from).collect()
    }
}
