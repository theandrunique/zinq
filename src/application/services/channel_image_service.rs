use std::sync::Arc;

use sha1::{Sha1, Digest};

use crate::{
    config::S3Config,
    infra::{ImageProcessor, S3Service},
};

const CHANNEL_IMAGE_MAX_SIZE: u32 = 512;

pub struct ChannelImageService {
    s3_service: Arc<dyn S3Service>,
    bucket_name: String,
}

impl ChannelImageService {
    pub fn new(s3_service: Arc<dyn S3Service>, s3_config: &S3Config) -> Self {
        Self {
            s3_service,
            bucket_name: s3_config.bucket_name.clone(),
        }
    }

    pub async fn upload_image(
        &self,
        chat_id: i64,
        image_data: &[u8],
    ) -> Result<String, anyhow::Error> {
        let processed = if ImageProcessor::is_gif(image_data) {
            ImageProcessor::process_gif(image_data, CHANNEL_IMAGE_MAX_SIZE)?
        } else {
            ImageProcessor::process_chat_image(image_data, CHANNEL_IMAGE_MAX_SIZE)?
        };

        let hash = compute_sha1_hex(image_data);
        let storage_key = format!("chats/{}/images/{}", chat_id, hash);

        self.s3_service
            .put_object(&storage_key, processed.data, &processed.content_type)
            .await?;

        Ok(storage_key)
    }
}

fn compute_sha1_hex(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}