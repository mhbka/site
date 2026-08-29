use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    routes::{
        error::{RouteError, RouteResult},
        users::is_author,
    },
    s3::UploadUrls,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/uploads", post(create_upload))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUploadRequest {
    post_id: Uuid,
    content_type: String,
}

/// Creates a one-time, short-lived URL for uploading a post image directly to
/// object storage. The caller never receives storage credentials.
async fn create_upload(
    State(app_state): State<AppState>,
    user: AuthUser,
    Json(request): Json<CreateUploadRequest>,
) -> RouteResult<Json<UploadUrls>> {
    if !is_author(&app_state.pool, user.id).await? {
        return Err(RouteError::forbidden("author access required"));
    }
    let owns_post = sqlx::query_scalar::<_, bool>(
        "select exists(select 1 from posts where id = $1 and author_id = $2 and deleted_at is null)",
    )
        .bind(request.post_id)
        .bind(user.id)
        .fetch_one(&app_state.pool)
        .await?;
    if !owns_post {
        return Err(RouteError::not_found("post not found"));
    }
    let urls = app_state
        .s3
        .generate_presigned_image_upload_url(&request.post_id.to_string(), &request.content_type)
        .await
        .map_err(|err| RouteError::S3(err.to_string()))?;
    Ok(Json(urls))
}
