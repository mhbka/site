use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a post.
///
/// Put very simply, draft = not displayed publicly, published = displayed publicly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "post_status", rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
}

/// Defines a blog post.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content_md: String,
    pub status: PostStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Summary of a post (mainly for listing posts).
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PostSummary {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub published_at: DateTime<Utc>,
}

/// Summary of a draft post for an author's private drafts listing.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DraftPostSummary {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

/// Accepts the fields required to create a draft post.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    pub title: String,
    pub content_md: String,
    pub slug: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Accepts optional fields to update an existing post.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content_md: Option<String>,
    pub thumbnail_url: Option<String>,
    pub slug: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[cfg(test)]
#[path = "../../tests/unit/models/posts.rs"]
mod tests;
