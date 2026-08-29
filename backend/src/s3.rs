use std::time::Duration;

use anyhow::anyhow;
use aws_config::Region;
use aws_sdk_s3::{self as s3, config::Credentials, presigning::PresigningConfig};
use serde::Serialize;
use uuid::Uuid;

const BLOGPOST_MEDIA_PRESIGNED_URL_EXPIRY: Duration = Duration::from_secs(60);

/// S3-related stuff.
#[derive(Debug, Clone)]
pub struct S3 {
    pub client: s3::Client,
    pub blogpost_bucket_name: String,
    pub blogpost_bucket_url: String,
}

impl S3 {
    pub async fn new(
        s3_account_id: String,
        s3_access_key_id: String,
        s3_access_key_secret: String,
        s3_blogpost_bucket_name: String,
        s3_blogpost_bucket_url: String,
    ) -> Self {
        let client = init_s3_client(s3_account_id, s3_access_key_id, s3_access_key_secret).await;
        Self {
            client,
            blogpost_bucket_name: s3_blogpost_bucket_name,
            blogpost_bucket_url: s3_blogpost_bucket_url,
        }
    }

    pub async fn generate_presigned_blogpost_media_upload_url(
        &self,
        post_id: &str,
        content_type: &str,
    ) -> Result<UploadUrls, anyhow::Error> {
        let extension = image_extension(content_type).ok_or(anyhow!("unsupported content type"))?;
        let key = format!("post-images/{}/{}.{}", post_id, Uuid::new_v4(), extension);
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

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrls {
    pub upload_url: String,
    pub public_url: String,
}

#[cfg(test)]
mod tests {
    use super::UploadUrls;

    #[test]
    fn serializes_upload_urls_as_camel_case() {
        let urls = UploadUrls {
            upload_url: "https://storage.example.test/upload".to_string(),
            public_url: "https://images.example.test/image.png".to_string(),
        };

        assert_eq!(
            serde_json::to_value(urls).unwrap(),
            serde_json::json!({
                "uploadUrl": "https://storage.example.test/upload",
                "publicUrl": "https://images.example.test/image.png",
            })
        );
    }
}

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

fn image_extension(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/avif" => Some("avif"),
        "image/gif" => Some("gif"),
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}
