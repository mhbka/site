use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::models::comments::{Comment, CreateCommentRequest, UpdateCommentRequest};
use crate::routes::error::{RouteError, RouteResult};
use crate::{auth::AuthUser, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/post/:post_id", get(list_comments).post(create_comment))
        .route(
            "/:id",
            axum::routing::put(update_comment).delete(delete_comment),
        )
}

/// GET /comments/post/:post_id — visible comments and deleted placeholders.
async fn list_comments(
    State(app_state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> RouteResult<Json<Vec<Comment>>> {
    let comments = sqlx::query_as::<_, Comment>(
        r#"
        select * from comments
        where post_id = $1 and (status = 'visible' or deleted_at is not null)
        order by created_at asc
        "#,
    )
    .bind(post_id)
    .fetch_all(&app_state.pool)
    .await?;

    Ok(Json(comments))
}

/// POST /comments/post/:post_id — requires auth (same Supabase user
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

    if let Some(parent_comment_id) = req.parent_comment_id {
        let parent_exists = sqlx::query_scalar::<_, bool>(
            "select exists(select 1 from comments where id = $1 and post_id = $2 and deleted_at is null)",
        )
        .bind(parent_comment_id)
        .bind(post_id)
        .fetch_one(&app_state.pool)
        .await?;

        if !parent_exists {
            return Err(RouteError::bad_request(
                "parent comment does not belong to this post",
            ));
        }
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

async fn update_comment(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCommentRequest>,
) -> RouteResult<Json<Comment>> {
    if req.body.trim().is_empty() {
        return Err(RouteError::bad_request("comment body is empty"));
    }

    let comment = sqlx::query_as::<_, Comment>(
        "update comments set body = $1 where id = $2 and author_id = $3 and deleted_at is null returning *",
    )
    .bind(req.body)
    .bind(id)
    .bind(user.id)
    .fetch_optional(&app_state.pool)
    .await?
    .ok_or(RouteError::not_found("comment not found"))?;

    Ok(Json(comment))
}

async fn delete_comment(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<StatusCode> {
    let deleted = sqlx::query(
        "update comments set body = '', deleted_at = now() where id = $1 and author_id = $2 and deleted_at is null",
    )
    .bind(id)
    .bind(user.id)
    .execute(&app_state.pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(RouteError::not_found("comment not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
