use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use tower::ServiceExt;
use uuid::Uuid;

use crate::models::pix::PixPage;

use super::CreateUploadResponse;

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
