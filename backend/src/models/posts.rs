use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "post_status", rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content_md: String,
    pub status: PostStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub thumbnail_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight shape for listing pages — skip the post body.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PostSummary {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content_md: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content_md: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::PostStatus;

    #[test]
    fn serializes_status_as_lowercase() {
        assert_eq!(
            serde_json::to_string(&PostStatus::Published).unwrap(),
            "\"published\""
        );
    }

    #[test]
    fn rejects_unknown_statuses() {
        assert!(serde_json::from_str::<PostStatus>("\"scheduled\"").is_err());
    }
}
