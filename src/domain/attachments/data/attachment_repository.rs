use async_trait::async_trait;

use crate::domain::attachments::Attachment;

#[async_trait]
pub trait AttachmentRepository: Send + Sync {
    async fn get_by_id(
        &self,
        chat_id: i64,
        attachment_id: i64,
    ) -> Result<Option<Attachment>, anyhow::Error>;

    async fn get_channel_attachments(
        &self,
        chat_id: i64,
        before_message_id: i64,
        limit: i32,
    ) -> Result<Vec<Attachment>, anyhow::Error>;

    async fn update_signed_urls(
        &self,
        attachments: Vec<Attachment>,
    ) -> Result<(), anyhow::Error>;
}
