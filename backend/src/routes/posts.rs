use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{AuthUser, OptionalAuthUser};
use crate::db;
use crate::models::posts::*;
use crate::routes::error::{RouteError, RouteResult};

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/:slug", get(get_post_by_slug))
        .route("/posts/id/:id", put(update_post).delete(delete_post))
        .route("/posts/id/:id/publish", post(publish_post))
}

/// GET /posts — published posts only, newest first. Paginate as needed.
async fn list_posts(
    State(pool): State<PgPool>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> RouteResult<Json<Vec<PostSummary>>> {
    let posts = sqlx::query_as::<_, PostSummary>(
        r#"
        select id, title, slug, thumbnail_url
        from posts
        where status = 'published' and published_at <= now() and deleted_at is null
        order by published_at desc
        limit 50
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(posts))
}

/// GET /posts/:slug — public read of a single published post.
async fn get_post_by_slug(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> RouteResult<Json<Post>> {
    let post = sqlx::query_as::<_, Post>(
        r#"
        select * from posts
        where slug = $1 and status = 'published' and published_at <= now() and deleted_at is null
        "#,
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await?
    .ok_or(RouteError::not_found("post not found"))?;

    Ok(Json(post))
}

/// POST /posts — creates a draft. Requires auth.
async fn create_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(req): Json<CreatePostRequest>,
) -> RouteResult<Json<Post>> {
    let slug = db::slugify(&req.title);
    let post = sqlx::query_as::<_, Post>(
        r#"
        insert into posts (author_id, title, slug, content_md)
        values ($1, $2, $3, $4)
        returning *
        "#,
    )
    .bind(user.id)
    .bind(&req.title)
    .bind(&slug)
    .bind(&req.content_md)
    .fetch_one(&pool)
    .await?;

    // Snapshot the first revision immediately.
    sqlx::query(
        "insert into post_revisions (post_id, title, content_md, created_by) values ($1, $2, $3, $4)",
    )
    .bind(post.id)
    .bind(&post.title)
    .bind(&post.content_md)
    .bind(user.id)
    .execute(&pool)
    .await?;

    Ok(Json(post))
}

/// PUT /posts/id/:id — edit a post (title/content/etc). Author-only.
async fn update_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> RouteResult<Json<Post>> {
    let existing =
        sqlx::query_as::<_, Post>("select * from posts where id = $1 and author_id = $2")
            .bind(id)
            .bind(user.id)
            .fetch_optional(&pool)
            .await?
            .ok_or(RouteError::not_found("post not found"))?;

    let title = req.title.unwrap_or(existing.title);
    let content_md = req.content_md.unwrap_or(existing.content_md);
    let thumbnail_url = req.thumbnail_url.or(existing.thumbnail_url);

    let post = sqlx::query_as::<_, Post>(
        r#"
        update posts set
          title = $1, content_md = $2, thumbnail_url = $3
        where id = $4 and author_id = $5
        returning *
        "#,
    )
    .bind(&title)
    .bind(&content_md)
    .bind(&thumbnail_url)
    .bind(id)
    .bind(user.id)
    .fetch_one(&pool)
    .await?;

    // Snapshot a revision on every save. Prune/collapse old ones later if the
    // table grows large — not a concern early on.
    sqlx::query(
        "insert into post_revisions (post_id, title, content_md, created_by) values ($1, $2, $3, $4)",
    )
    .bind(post.id)
    .bind(&post.title)
    .bind(&post.content_md)
    .bind(user.id)
    .execute(&pool)
    .await?;

    Ok(Json(post))
}

/// POST /posts/id/:id/publish — publishes immediately.
async fn publish_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<Json<Post>> {
    let post = sqlx::query_as::<_, Post>(
        r#"
        update posts set status = $1, published_at = now()
        where id = $2 and author_id = $3
        returning *
        "#,
    )
    .bind(PostStatus::Published)
    .bind(id)
    .bind(user.id)
    .fetch_optional(&pool)
    .await?
    .ok_or(RouteError::not_found("post not found"))?;

    Ok(Json(post))
}

/// DELETE /posts/id/:id — soft delete.
async fn delete_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<StatusCode> {
    sqlx::query("update posts set deleted_at = now() where id = $1 and author_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
