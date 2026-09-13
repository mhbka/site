use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use tower::ServiceExt;
use uuid::Uuid;

use crate::models::pix::PixPage;

use super::{normalize_tags, CreateUploadResponse, TagOperation, UpdateTagsRequest};

#[test]
fn serializes_upload_response_as_camel_case() {
    let response = CreateUploadResponse {
        image_id: Uuid::nil(),
        upload_url: "upload".into(),
        public_url: "public".into(),
    };
    assert_eq!(
        serde_json::to_value(response).unwrap(),
        serde_json::json!({ "imageId": Uuid::nil(), "uploadUrl": "upload", "publicUrl": "public" })
    );
}

#[test]
fn serializes_pix_page_pagination_state_as_camel_case() {
    let page = PixPage {
        images: vec![],
        next_before: None,
        has_more: false,
    };
    assert_eq!(
        serde_json::to_value(page).unwrap(),
        serde_json::json!({ "images": [], "nextBefore": null, "hasMore": false })
    );
}

#[test]
fn deserializes_bulk_tag_update_request() {
    let request: UpdateTagsRequest = serde_json::from_value(serde_json::json!({
        "imageIds": [Uuid::nil()],
        "operation": "add",
        "tags": ["cats"]
    }))
    .unwrap();

    assert_eq!(request.image_ids, vec![Uuid::nil()]);
    assert!(matches!(request.operation, TagOperation::Add));
    assert_eq!(request.tags, vec!["cats"]);
}

#[test]
fn normalizes_bulk_tag_update_tags() {
    assert_eq!(
        normalize_tags(vec![" Cats ".into(), "cats".into(), "".into()]),
        vec!["cats"]
    );
}

#[tokio::test]
async fn complete_upload_route_matches_a_uuid_path() {
    let router = Router::new().route(
        "/uploads/{id}/complete",
        post(|| async { StatusCode::NO_CONTENT }),
    );
    let response = router
        .oneshot(
            Request::post("/uploads/f2c6e5e4-29ef-4dce-896e-3ea8a2e9507d/complete")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
