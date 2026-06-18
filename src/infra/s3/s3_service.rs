use std::sync::Arc;

use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;

use crate::config::S3Config;

#[derive(Debug, Clone)]
pub struct S3ObjectMetadata {
    pub key: String,
    pub content_type: String,
    pub content_length: i64,
}

#[async_trait]
pub trait S3Service: Send + Sync {
    async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), anyhow::Error>;

    async fn generate_presigned_upload_url(
        &self,
        key: &str,
        expires_in: i64,
        content_length: i64,
    ) -> Result<String, anyhow::Error>;

    async fn generate_presigned_download_url(
        &self,
        key: &str,
        expires_in: i64,
    ) -> Result<String, anyhow::Error>;

    async fn delete_object(&self, key: &str) -> Result<(), anyhow::Error>;

    async fn get_object_metadata(
        &self,
        key: &str,
    ) -> Result<Option<S3ObjectMetadata>, anyhow::Error>;

    async fn is_object_exists(&self, key: &str) -> Result<bool, anyhow::Error>;

    async fn create_bucket(&self, bucket_name: &str) -> Result<(), anyhow::Error>;
}

pub struct AwsS3Service {
    client: Client,
    bucket_name: String,
}

impl AwsS3Service {
    pub async fn new(config: &S3Config) -> Self {
        use aws_sdk_s3::config::SharedCredentialsProvider;

        let credentials = aws_sdk_s3::config::Credentials::new(
            config.access_key.clone(),
            config.secret_key.clone(),
            None,
            None,
            "static",
        );

        let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest())
            .await
            .to_builder()
            .credentials_provider(SharedCredentialsProvider::new(credentials))
            .region(aws_sdk_s3::config::Region::new(config.region.clone()))
            .build();

        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(config.force_path_style);

        s3_config_builder.set_endpoint_url(Some(config.service_url.clone()));

        let s3_config = s3_config_builder.build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket_name: config.bucket_name.clone(),
        }
    }
}

#[async_trait]
impl S3Service for AwsS3Service {
    async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), anyhow::Error> {
        use aws_sdk_s3::primitives::ByteStream;

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(ByteStream::from(body))
            .content_type(content_type)
            .send()
            .await?;

        Ok(())
    }

    async fn generate_presigned_upload_url(
        &self,
        key: &str,
        expires_after_secs: i64,
        content_length: i64,
    ) -> Result<String, anyhow::Error> {
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .content_length(content_length)
            .presigned(
                PresigningConfig::builder()
                    .expires_in(std::time::Duration::from_secs(expires_after_secs as u64))
                    .build()?,
            )
            .await?;

        Ok(presigned.uri().to_string())
    }

    async fn generate_presigned_download_url(
        &self,
        key: &str,
        expires_after_secs: i64,
    ) -> Result<String, anyhow::Error> {
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .presigned(
                PresigningConfig::builder()
                    .expires_in(std::time::Duration::from_secs(expires_after_secs as u64))
                    .build()?,
            )
            .await?;

        Ok(presigned.uri().to_string())
    }

    async fn delete_object(&self, key: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    async fn get_object_metadata(
        &self,
        key: &str,
    ) -> Result<Option<S3ObjectMetadata>, anyhow::Error> {
        let result = self
            .client
            .head_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await;

        match result {
            Ok(response) => {
                let content_type = response
                    .content_type()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                let content_length = response.content_length().unwrap_or(0);

                Ok(Some(S3ObjectMetadata {
                    key: key.to_string(),
                    content_type,
                    content_length,
                }))
            }
            Err(e) => {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("not found")
                    || err_str.contains("nosuchkey")
                    || err_str.contains("404")
                    || err_str.contains("no such key")
                    || err_str.contains("service error")
                {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("S3 error: {}", e))
                }
            }
        }
    }

    async fn is_object_exists(&self, key: &str) -> Result<bool, anyhow::Error> {
        Ok(self.get_object_metadata(key).await?.is_some())
    }

    async fn create_bucket(&self, bucket_name: &str) -> Result<(), anyhow::Error> {
        use aws_sdk_s3::types::BucketLocationConstraint;

        let constraint = BucketLocationConstraint::from(self.bucket_name.as_str());

        self.client
            .create_bucket()
            .bucket(bucket_name)
            .set_create_bucket_configuration(Some(
                aws_sdk_s3::types::CreateBucketConfiguration::builder()
                    .location_constraint(constraint)
                    .build(),
            ))
            .send()
            .await?;

        Ok(())
    }
}
