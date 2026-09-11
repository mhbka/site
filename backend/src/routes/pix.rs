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

/// Builds the Pix gallery API routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_images))
        .route("/tags", get(list_tags))
        .route("/uploads", post(create_upload))
        .route("/uploads/{id}/complete", post(complete_upload))
}

/// Accepts optional filters for a cursor-paginated gallery query.
#[derive(Debug, Deserialize)]
struct ListImagesQuery {
    limit: Option<i64>,
    before: Option<Uuid>,
    tag: Option<String>,
}

/// Returns a page of completed Pix images.
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

/// Accepts metadata for a new Pix upload.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUploadRequest {
    content_type: String,
    tags: Vec<String>,
}

/// Returns the pending image ID and its upload URLs.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateUploadResponse {
    image_id: Uuid,
    upload_url: String,
    public_url: String,
}

/// Creates a pending Pix record and its direct upload URL.
async fn create_upload(
    State(app_state): State<AppState>,
    user: AuthUser,
    Json(request): Json<CreateUploadRequest>,
) -> RouteResult<Json<CreateUploadResponse>> {
    if !is_pix(&app_state.pool, user.id).await? {
        tracing::info!(user_id = %user.id, "Pix upload denied for user without access");
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
    .bind(&request.content_type)
    .bind(&request.tags)
    .execute(&app_state.pool)
    .await?;
    tracing::info!(image_id = %image_id, user_id = %user.id, content_type = %request.content_type, "Pix upload created");
    Ok(Json(CreateUploadResponse {
        image_id,
        upload_url,
        public_url,
    }))
}

/// Marks a verified object-storage upload as complete.
async fn complete_upload(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<Json<Pix>> {
    if !is_pix(&app_state.pool, user.id).await? {
        tracing::info!(user_id = %user.id, "Pix completion denied for user without access");
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
        tracing::info!(image_id = %image.id, user_id = %user.id, "Pix completion attempted before storage upload finished");
        return Err(RouteError::bad_request("image upload is incomplete"));
    }
    let image = sqlx::query_as::<_, Pix>(
        "update pix set uploaded_at = now() where id = $1
         returning id, public_url, tags, created_at",
    )
    .bind(image.id)
    .fetch_one(&app_state.pool)
    .await?;
    tracing::info!(image_id = %image.id, user_id = %user.id, "Pix upload completed");
    Ok(Json(image))
}

/// Returns the tags used by completed Pix images.
async fn list_tags(
    State(app_state): State<AppState>,
) -> RouteResult<Json<Vec<crate::models::tags::TagSummary>>> {
    Ok(Json(sqlx::query_as::<_, crate::models::tags::TagSummary>(
        "select tag, count(*)::bigint as count from pix cross join lateral unnest(tags) as tag where uploaded_at is not null group by tag order by count desc, tag asc",
    ).fetch_all(&app_state.pool).await?))
}

/// Holds the stored object key for a pending Pix image.
#[derive(sqlx::FromRow)]
struct PendingImage {
    id: Uuid,
    bucket_path: String,
}

#[cfg(test)]
#[path = "../../tests/unit/routes/pix.rs"]
mod tests;
