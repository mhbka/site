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
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    pub title: String,
    pub content_md: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content_md: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{DraftPostSummary, PostStatus, PostSummary, UpdatePostRequest};
    use chrono::Utc;
    use uuid::Uuid;

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

    #[test]
    fn serializes_post_fields_as_camel_case() {
        let post = PostSummary {
            id: Uuid::nil(),
            title: "Hello".to_string(),
            slug: "hello".to_string(),
            thumbnail_url: Some("https://example.test/image.png".to_string()),
            published_at: Utc::now(),
        };

        let json = serde_json::to_value(post).unwrap();
        assert_eq!(json["thumbnailUrl"], "https://example.test/image.png");
        assert!(json.get("thumbnail_url").is_none());
    }

    #[test]
    fn serializes_draft_summary_fields_as_camel_case() {
        let post = DraftPostSummary {
            id: Uuid::nil(),
            title: "Draft".to_string(),
            slug: "draft".to_string(),
            thumbnail_url: None,
            updated_at: Utc::now(),
        };

        let json = serde_json::to_value(post).unwrap();
        assert!(json.get("updatedAt").is_some());
        assert!(json.get("updated_at").is_none());
    }

    #[test]
    fn deserializes_post_requests_from_camel_case() {
        let request: UpdatePostRequest = serde_json::from_value(serde_json::json!({
            "contentMd": "Updated content",
            "thumbnailUrl": "https://example.test/image.png"
        }))
        .unwrap();

        assert_eq!(request.content_md.as_deref(), Some("Updated content"));
        assert_eq!(
            request.thumbnail_url.as_deref(),
            Some("https://example.test/image.png")
        );
    }
}
