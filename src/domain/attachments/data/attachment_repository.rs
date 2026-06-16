use async_trait::async_trait;

use crate::domain::attachments::Attachment;

#[async_trait]
pub trait AttachmentRepository: Send + Sync {
    async fn save(&self, attachment: &Attachment) -> Result<(), anyhow::Error>;
    async fn bulk_save(&self, attachments: &[Attachment]) -> Result<(), anyhow::Error>;

    async fn get_by_id(
        &self,
        chat_id: i64,
        attachment_id: i64,
    ) -> Result<Option<Attachment>, anyhow::Error>;

    async fn get_chat_attachments(
        &self,
        chat_id: i64,
        before_message_id: i64,
        limit: i32,
    ) -> Result<Vec<Attachment>, anyhow::Error>;

    async fn get_by_message_ids(
        &self,
        chat_id: i64,
        message_ids: &[i64],
    ) -> Result<Vec<Attachment>, anyhow::Error>;
}
