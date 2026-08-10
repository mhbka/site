use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{auth::AuthUser, state::AppState};
use crate::models::comments::{Comment, CreateCommentRequest};
use crate::routes::error::{RouteError, RouteResult};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/post/:post_id",
        get(list_comments).post(create_comment),
    )
}

/// GET /posts/id/:post_id/comments — visible comments only.
async fn list_comments(
    State(app_state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> RouteResult<Json<Vec<Comment>>> {
    let comments = sqlx::query_as::<_, Comment>(
        r#"
        select * from comments
        where post_id = $1 and status = 'visible'
        order by created_at asc
        "#,
    )
    .bind(post_id)
    .fetch_all(&app_state.pool)
    .await?;

    Ok(Json(comments))
}

/// POST /posts/id/:post_id/comments — requires auth (same Supabase user
/// system as the rest of the site).
async fn create_comment(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> RouteResult<Json<Comment>> {
    if req.body.trim().is_empty() {
        return Err(RouteError::bad_request("comment body is empty"));
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
    .fetch_one(&app_state.pool)
    .await?;

    Ok(Json(comment))
}
