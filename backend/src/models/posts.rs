use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub content_md: String,
    pub content_html: String,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub seo_description: Option<String>,
    pub og_image_url: Option<String>,
    pub reading_time_min: Option<i32>,
    pub preview_token: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight shape for listing pages — skip full content_html/content_md.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PostSummary {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub reading_time_min: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content_md: String,
    pub excerpt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content_md: Option<String>,
    pub excerpt: Option<String>,
    pub seo_description: Option<String>,
    pub og_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublishPostRequest {
    /// Publish immediately if None, or schedule for the future.
    pub published_at: Option<DateTime<Utc>>,
}