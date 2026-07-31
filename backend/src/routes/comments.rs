use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::models::comments::{Comment, CreateCommentRequest};

pub fn router() -> Router<PgPool> {
    Router::new().route(
        "/posts/id/:post_id/comments",
        get(list_comments).post(create_comment),
    )
}

/// GET /posts/id/:post_id/comments — visible comments only (RLS also
/// enforces this, but filtering here avoids relying solely on it).
async fn list_comments(
    State(pool): State<PgPool>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<Vec<Comment>>, (StatusCode, String)> {
    let comments = sqlx::query_as::<_, Comment>(
        r#"
        select * from comments
        where post_id = $1 and status = 'visible'
        order by created_at asc
        "#,
    )
    .bind(post_id)
    .fetch_all(&pool)
    .await
    .map_err(internal_err)?;

    Ok(Json(comments))
}

/// POST /posts/id/:post_id/comments — requires auth (same Supabase user
/// system as the rest of the site).
async fn create_comment(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, (StatusCode, String)> {
    if req.body.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "comment body is empty".into()));
    }

    let comment = sqlx::query_as::<_, Comment>(
        r#"
        insert into comments (post_id, author_id, parent_comment_id, body)
        values ($1, $2, $3, $4)
        returning *
        "#,
    )
    .bind(post_id)
    .bind(user.id)
    .bind(req.parent_comment_id)
    .bind(&req.body)
    .fetch_one(&pool)
    .await
    .map_err(internal_err)?;

    Ok(Json(comment))
}

fn internal_err(e: sqlx::Error) -> (StatusCode, String) {
    tracing::error!("db error: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
}