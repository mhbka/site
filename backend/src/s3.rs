use std::time::Duration;

use anyhow::anyhow;
use aws_config::Region;
use aws_sdk_s3::{self as s3, config::Credentials, presigning::PresigningConfig};
use serde::Serialize;
use uuid::Uuid;

const BLOGPOST_MEDIA_PRESIGNED_URL_EXPIRY: Duration = Duration::from_secs(60);

/// Holds the object-storage client and bucket configuration.
#[derive(Debug, Clone)]
pub struct S3 {
    pub client: s3::Client,
    pub blogpost_bucket_name: String,
    pub blogpost_bucket_url: String,
    pub pix_bucket_name: String,
    pub pix_bucket_url: String,
}

impl S3 {
    /// Builds an object-storage client from the configured R2 credentials.
    pub async fn new(
        s3_account_id: String,
        s3_access_key_id: String,
        s3_access_key_secret: String,
        s3_blogpost_bucket_name: String,
        s3_blogpost_bucket_url: String,
        s3_pix_bucket_name: String,
        s3_pix_bucket_url: String,
    ) -> Self {
        tracing::info!("initializing object storage client");
        let client = init_s3_client(s3_account_id, s3_access_key_id, s3_access_key_secret).await;
        Self {
            client,
            blogpost_bucket_name: s3_blogpost_bucket_name,
            blogpost_bucket_url: s3_blogpost_bucket_url,
            pix_bucket_name: s3_pix_bucket_name,
            pix_bucket_url: s3_pix_bucket_url,
        }
    }

    /// Creates upload and public URLs for a pending Pix image.
    pub async fn generate_presigned_pix_upload_url(
        &self,
        image_id: Uuid,
        content_type: &str,
    ) -> Result<(String, UploadUrls), anyhow::Error> {
        let extension = image_extension(content_type).ok_or(anyhow!("unsupported content type"))?;
        let key = format!("images/{image_id}.{extension}");
        tracing::info!(%image_id, %content_type, "creating Pix upload URL");
        let presigning_config = PresigningConfig::expires_in(BLOGPOST_MEDIA_PRESIGNED_URL_EXPIRY)?;
        let upload_url = self
            .client
            .put_object()
            .bucket(&self.pix_bucket_name)
            .key(&key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map(|r| r.uri().to_string())?;
        Ok((
            key.clone(),
            UploadUrls {
                upload_url,
                public_url: format!("{}/{}", self.pix_bucket_url.trim_end_matches('/'), key),
            },
        ))
    }

    /// Checks whether an uploaded Pix object exists in storage.
    pub async fn pix_exists(&self, key: &str) -> Result<bool, anyhow::Error> {
        match self
            .client
            .head_object()
            .bucket(&self.pix_bucket_name)
            .key(key)
            .send()
            .await
        {
            Ok(_) => {
                tracing::debug!(%key, "Pix object found in storage");
                Ok(true)
            }
            Err(error)
                if error
                    .as_service_error()
                    .is_some_and(|service| service.is_not_found()) =>
            {
                tracing::info!(%key, "Pix object not yet present in storage");
                Ok(false)
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Creates upload and public URLs for a blog-post image.
    pub async fn generate_presigned_blogpost_media_upload_url(
        &self,
        post_id: &str,
        content_type: &str,
    ) -> Result<UploadUrls, anyhow::Error> {
        let extension = image_extension(content_type).ok_or(anyhow!("unsupported content type"))?;
        let key = format!("post-images/{}/{}.{}", post_id, Uuid::new_v4(), extension);
        tracing::info!(%post_id, %content_type, "creating post-media upload URL");
        let presigning_config = PresigningConfig::expires_in(BLOGPOST_MEDIA_PRESIGNED_URL_EXPIRY)?;
        let url = self
            .client
            .put_object()
            .bucket(&self.blogpost_bucket_name)
            .key(&key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map(|r| r.uri().to_string())?;
        Ok(UploadUrls {
            upload_url: url,
            public_url: format!("{}/{}", self.blogpost_bucket_url.trim_end_matches('/'), key),
        })
    }
}

/// Contains the direct upload URL and the eventual public asset URL.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrls {
    pub upload_url: String,
    pub public_url: String,
}

#[cfg(test)]
#[path = "../tests/unit/s3.rs"]
mod tests;

/// Initializes an S3-compatible client for Cloudflare R2.
async fn init_s3_client(
    s3_account_id: String,
    s3_access_key_id: String,
    s3_access_key_secret: String,
) -> s3::Client {
    let config = aws_config::from_env()
        .endpoint_url(format!("https://{s3_account_id}.r2.cloudflarestorage.com"))
        .credentials_provider(Credentials::new(
            s3_access_key_id,
            s3_access_key_secret,
            None,
            None,
            "R2",
        ))
        .region(Region::new("auto"))
        .load()
        .await;
    s3::Client::new(&config)
}

/// Maps an accepted image MIME type to its filename extension.
pub fn image_extension(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/avif" => Some("avif"),
        "image/gif" => Some("gif"),
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}
