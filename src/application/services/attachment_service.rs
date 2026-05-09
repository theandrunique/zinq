use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;

use crate::{
    config::S3Config,
    domain::attachments::{Attachment, CreateAttachmentRequest},
    error::Error,
    infra::{IdGenerator, S3Service, s3::S3ObjectMetadata},
};

#[derive(Debug, Clone)]
pub struct UploadUrl {
    pub storage_key: String,
    pub upload_url: String,
}

pub struct ParseUploadFilenameResult {
    pub chat_id: i64,
    pub attachment_id: i64,
    pub filename: String,
}

pub struct AttachmentService {
    s3_service: Arc<dyn S3Service>,
    id_gen: Arc<dyn IdGenerator>,
    bucket_name: String,
}

impl AttachmentService {
    pub fn new(
        s3_service: Arc<dyn S3Service>,
        id_gen: Arc<dyn IdGenerator>,
        s3_config: &S3Config,
    ) -> Self {
        Self {
            s3_service,
            id_gen,
            bucket_name: s3_config.bucket_name.clone(),
        }
    }

    pub async fn generate_upload_url(
        &self,
        size: i64,
        chat_id: i64,
        filename: &str,
    ) -> Result<UploadUrl, anyhow::Error> {
        let attachment_id = self.id_gen.gen_id().await;
        let storage_key = self.generate_storage_key(filename, chat_id, attachment_id);

        let upload_url = self
            .s3_service
            .generate_presigned_upload_url(&storage_key, 3600, size)
            .await?;

        Ok(UploadUrl {
            storage_key,
            upload_url,
        })
    }

    pub async fn validate_and_create_attachment(
        &self,
        message_id: i64,
        uploaded_filename: &str,
        filename: &str,
    ) -> Result<Attachment, Error> {
        let parsed = self.parse_storage_key(uploaded_filename).ok_or_else(|| {
            Error::AttachmentInvalidUploadFilename {
                upload_filename: uploaded_filename.to_string(),
            }
        })?;

        let object_meta = self
            .s3_service
            .get_object_metadata(uploaded_filename)
            .await?
            .ok_or_else(|| Error::AttachmentObjectNotFound {
                upload_filename: uploaded_filename.to_string(),
            })?;

        let (content_type, size) = extract_object_info(&object_meta);

        Ok(Attachment::new(CreateAttachmentRequest {
            id: parsed.attachment_id,
            message_id: message_id,
            chat_id: parsed.chat_id,
            filename: filename.to_string(),
            content_type: content_type,
            size: size,
            storage_key: uploaded_filename.to_string(),
        }))
    }

    pub async fn delete_object(&self, storage_key: &str) -> Result<(), anyhow::Error> {
        self.s3_service.delete_object(storage_key).await
    }

    pub async fn is_object_exists(&self, storage_key: &str) -> Result<bool, anyhow::Error> {
        self.s3_service.is_object_exists(storage_key).await
    }

    fn generate_storage_key(&self, filename: &str, chat_id: i64, attachment_id: i64) -> String {
        format!("attachments/{}/{}/{}", chat_id, attachment_id, filename)
    }

    pub fn parse_storage_key(&self, key: &str) -> Option<ParseUploadFilenameResult> {
        static UPLOAD_FILENAME_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new(r"^attachments/(?P<chat_id>\d+)/(?P<attachment_id>\d+)/(?P<filename>.+)$")
                .unwrap()
        });

        let caps = UPLOAD_FILENAME_REGEX.captures(key)?;

        let chat_id: i64 = caps.name("chat_id")?.as_str().parse().ok()?;
        let attachment_id: i64 = caps.name("attachment_id")?.as_str().parse().ok()?;
        let filename = caps.name("filename")?.as_str().to_string();

        Some(ParseUploadFilenameResult {
            chat_id,
            attachment_id,
            filename,
        })
    }
}

fn extract_object_info(obj: &S3ObjectMetadata) -> (String, i64) {
    let key = &obj.key;
    let content_type = if obj.content_type.is_empty() {
        if key.ends_with(".png") {
            "image/png".to_string()
        } else if key.ends_with(".jpg") || key.ends_with(".jpeg") {
            "image/jpeg".to_string()
        } else if key.ends_with(".gif") {
            "image/gif".to_string()
        } else if key.ends_with(".webp") {
            "image/webp".to_string()
        } else {
            "application/octet-stream".to_string()
        }
    } else {
        obj.content_type.clone()
    };

    (content_type, obj.content_length)
}
