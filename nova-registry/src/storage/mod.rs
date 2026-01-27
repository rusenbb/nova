//! S3/R2 storage client.

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

/// S3 client wrapper.
#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
    public_url: String,
}

impl S3Client {
    /// Create client from environment variables.
    ///
    /// Required env vars:
    /// - S3_BUCKET: Bucket name
    /// - S3_PUBLIC_URL: Public URL prefix for downloads
    ///
    /// Optional (uses AWS defaults if not set):
    /// - S3_ENDPOINT: Custom endpoint (e.g., for Cloudflare R2)
    /// - AWS_ACCESS_KEY_ID: Access key
    /// - AWS_SECRET_ACCESS_KEY: Secret key
    /// - AWS_REGION: Region (default: auto)
    pub async fn from_env() -> anyhow::Result<Self> {
        let bucket =
            std::env::var("S3_BUCKET").unwrap_or_else(|_| "nova-extensions".to_string());
        let public_url = std::env::var("S3_PUBLIC_URL")
            .unwrap_or_else(|_| format!("https://{}.s3.amazonaws.com", bucket));

        // Build AWS config
        let mut config_builder = aws_config::from_env();

        // Custom endpoint for R2 or MinIO
        if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
            config_builder = config_builder.endpoint_url(&endpoint);
        }

        let config = config_builder.load().await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            bucket,
            public_url,
        })
    }

    /// Upload a file and return the public URL.
    pub async fn upload(&self, key: &str, data: &[u8]) -> Result<String, crate::api::ApiError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()))
            .content_type("application/gzip")
            .send()
            .await
            .map_err(|e| {
                tracing::error!("S3 upload error: {:?}", e);
                crate::api::ApiError::Internal("Failed to upload file".to_string())
            })?;

        Ok(format!("{}/{}", self.public_url, key))
    }

    /// Delete a file.
    #[allow(dead_code)]
    pub async fn delete(&self, key: &str) -> Result<(), crate::api::ApiError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("S3 delete error: {:?}", e);
                crate::api::ApiError::Internal("Failed to delete file".to_string())
            })?;

        Ok(())
    }

    /// Check if a file exists.
    #[allow(dead_code)]
    pub async fn exists(&self, key: &str) -> Result<bool, crate::api::ApiError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("404") || e.to_string().contains("NotFound") {
                    Ok(false)
                } else {
                    tracing::error!("S3 head error: {:?}", e);
                    Err(crate::api::ApiError::Internal(
                        "Failed to check file".to_string(),
                    ))
                }
            }
        }
    }
}
