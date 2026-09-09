use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    models::pix::{Pix, PixPage},
    routes::{
        error::{RouteError, RouteResult},
        users::is_pix,
    },
    s3::{image_extension, UploadUrls},
    state::AppState,
};

const DEFAULT_PAGE_SIZE: i64 = 60;
const MAX_PAGE_SIZE: i64 = 100;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_images))
        .route("/tags", get(list_tags))
        .route("/uploads", post(create_upload))
        .route("/uploads/{id}/complete", post(complete_upload))
}

#[derive(Debug, Deserialize)]
struct ListImagesQuery {
    limit: Option<i64>,
    before: Option<Uuid>,
    tag: Option<String>,
}

async fn list_images(
    State(app_state): State<AppState>,
    Query(query): Query<ListImagesQuery>,
) -> RouteResult<Json<PixPage>> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let images = sqlx::query_as::<_, Pix>(
        "select id, public_url, tags, created_at
         from pix
         where uploaded_at is not null
           and ($2::text is null or tags @> array[$2])
           and ($1::uuid is null or (created_at, id) < (select created_at, id from pix where id = $1))
         order by created_at desc, id desc
         limit $3",
    )
    .bind(query.before)
    .bind(query.tag)
    .bind(limit + 1)
    .fetch_all(&app_state.pool)
    .await?;
    let has_more = images.len() as i64 > limit;
    let next_before = has_more.then(|| images[limit as usize - 1].id);
    Ok(Json(PixPage {
        images: images.into_iter().take(limit as usize).collect(),
        next_before,
        has_more,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUploadRequest {
    content_type: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateUploadResponse {
    image_id: Uuid,
    upload_url: String,
    public_url: String,
}

async fn create_upload(
    State(app_state): State<AppState>,
    user: AuthUser,
    Json(request): Json<CreateUploadRequest>,
) -> RouteResult<Json<CreateUploadResponse>> {
    if !is_pix(&app_state.pool, user.id).await? {
        return Err(RouteError::forbidden("pix access required"));
    }
    if image_extension(&request.content_type).is_none() {
        return Err(RouteError::bad_request("unsupported image type"));
    }
    if request.tags.is_empty() {
        return Err(RouteError::bad_request("at least one tag is required"));
    }
    let image_id = Uuid::new_v4();
    let (
        bucket_path,
        UploadUrls {
            upload_url,
            public_url,
        },
    ) = app_state
        .s3
        .generate_presigned_pix_upload_url(image_id, &request.content_type)
        .await
        .map_err(|error| RouteError::S3(error.to_string()))?;
    sqlx::query(
        "insert into pix (id, uploader_id, bucket_path, public_url, content_type, tags)
         values ($1, $2, $3, $4, $5, $6)",
    )
    .bind(image_id)
    .bind(user.id)
    .bind(bucket_path)
    .bind(&public_url)
    .bind(request.content_type)
    .bind(request.tags)
    .execute(&app_state.pool)
    .await?;
    Ok(Json(CreateUploadResponse {
        image_id,
        upload_url,
        public_url,
    }))
}

async fn complete_upload(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<Json<Pix>> {
    if !is_pix(&app_state.pool, user.id).await? {
        return Err(RouteError::forbidden("pix access required"));
    }
    let image = sqlx::query_as::<_, PendingImage>(
        "select id, bucket_path from pix where id = $1 and uploader_id = $2 and uploaded_at is null",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&app_state.pool)
    .await?
    .ok_or(RouteError::not_found("pending image not found"))?;
    if !app_state
        .s3
        .pix_exists(&image.bucket_path)
        .await
        .map_err(|error| RouteError::S3(error.to_string()))?
    {
        return Err(RouteError::bad_request("image upload is incomplete"));
    }
    let image = sqlx::query_as::<_, Pix>(
        "update pix set uploaded_at = now() where id = $1
         returning id, public_url, tags, created_at",
    )
    .bind(image.id)
    .fetch_one(&app_state.pool)
    .await?;
    Ok(Json(image))
}

async fn list_tags(
    State(app_state): State<AppState>,
) -> RouteResult<Json<Vec<crate::models::tags::TagSummary>>> {
    Ok(Json(sqlx::query_as::<_, crate::models::tags::TagSummary>(
        "select tag, count(*)::bigint as count from pix cross join lateral unnest(tags) as tag where uploaded_at is not null group by tag order by count desc, tag asc",
    ).fetch_all(&app_state.pool).await?))
}

#[derive(sqlx::FromRow)]
struct PendingImage {
    id: Uuid,
    bucket_path: String,
}

#[cfg(test)]
mod tests {
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
}
