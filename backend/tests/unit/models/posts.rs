use super::{CreatePostRequest, DraftPostSummary, PostStatus, PostSummary, UpdatePostRequest};
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
        tags: vec!["rust".to_string()],
        published_at: Utc::now(),
    };

    let json = serde_json::to_value(post).unwrap();
    assert_eq!(json["thumbnailUrl"], "https://example.test/image.png");
    assert_eq!(json["tags"], serde_json::json!(["rust"]));
    assert!(json.get("thumbnail_url").is_none());
}

#[test]
fn serializes_draft_summary_fields_as_camel_case() {
    let post = DraftPostSummary {
        id: Uuid::nil(),
        title: "Draft".to_string(),
        slug: "draft".to_string(),
        thumbnail_url: None,
        tags: vec!["astro".to_string()],
        updated_at: Utc::now(),
    };

    let json = serde_json::to_value(post).unwrap();
    assert!(json.get("updatedAt").is_some());
    assert!(json.get("updated_at").is_none());
    assert_eq!(json["tags"], serde_json::json!(["astro"]));
}

#[test]
fn deserializes_post_requests_from_camel_case() {
    let request: UpdatePostRequest = serde_json::from_value(serde_json::json!({
        "contentMd": "Updated content",
        "thumbnailUrl": "https://example.test/image.png",
        "tags": ["rust"]
    }))
    .unwrap();

    assert_eq!(request.content_md.as_deref(), Some("Updated content"));
    assert_eq!(
        request.thumbnail_url.as_deref(),
        Some("https://example.test/image.png")
    );
    assert_eq!(request.slug, None);
    assert_eq!(request.tags, Some(vec!["rust".to_string()]));
}

#[test]
fn deserializes_optional_slug_from_camel_case_requests() {
    let request: CreatePostRequest = serde_json::from_value(serde_json::json!({
        "title": "Draft",
        "contentMd": "Content",
        "slug": "custom-draft",
        "tags": ["astro"]
    }))
    .unwrap();

    assert_eq!(request.slug.as_deref(), Some("custom-draft"));
    assert_eq!(request.tags, Some(vec!["astro".to_string()]));
}
