use std::sync::Arc;

use sha1::{Sha1, Digest};

use crate::{
    config::S3Config,
    domain::auth::User,
    infra::{ImageProcessor, S3Service},
};

const AVATAR_MAX_SIZE: u32 = 512;

pub struct AvatarService {
    s3_service: Arc<dyn S3Service>,
    bucket_name: String,
}

impl AvatarService {
    pub fn new(s3_service: Arc<dyn S3Service>, s3_config: &S3Config) -> Self {
        Self {
            s3_service,
            bucket_name: s3_config.bucket_name.clone(),
        }
    }

    pub async fn upload_avatar(&self, user: &User, image_data: &[u8]) -> Result<String, anyhow::Error> {
        let processed = if ImageProcessor::is_gif(image_data) {
            ImageProcessor::process_gif(image_data, AVATAR_MAX_SIZE)?
        } else {
            ImageProcessor::process_avatar(image_data, AVATAR_MAX_SIZE)?
        };

        let hash = compute_sha1_hex(image_data);
        let storage_key = format!("avatars/{}/{}", user.id, hash);

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