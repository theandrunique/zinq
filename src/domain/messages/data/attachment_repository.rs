use async_trait::async_trait;

use crate::domain::messages::attachment::Attachment;

#[async_trait]
pub trait AttachmentRepository: Send + Sync {
    async fn get_by_id(chat_id: i64, attachment_id: i64) -> Option<Attachment>;
    async fn get_channel_attachments(
        chat_id: i64,
        before_message_id: i64,
        limit: i32,
    ) -> Vec<Attachment>;
    async fn update_pre_signed_urls(chat_id: i64, attachments: Vec<Attachment>);
}
