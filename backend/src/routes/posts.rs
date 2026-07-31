use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{AuthUser, OptionalAuthUser};
use crate::db;
use crate::models::posts::*;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/:slug", get(get_post_by_slug))
        .route("/posts/id/:id", put(update_post).delete(delete_post))
        .route("/posts/id/:id/publish", post(publish_post))
        .route("/posts/preview/:token", get(get_post_by_preview_token))
}

/// GET /posts — published posts only, newest first. Paginate as needed.
async fn list_posts(
    State(pool): State<PgPool>,
    OptionalAuthUser(_user): OptionalAuthUser,
) -> Result<Json<Vec<PostSummary>>, (StatusCode, String)> {
    let posts = sqlx::query_as::<_, PostSummary>(
        r#"
        select id, title, slug, excerpt, published_at, reading_time_min
        from posts
        where status = 'published' and published_at <= now() and deleted_at is null
        order by published_at desc
        limit 50
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_err)?;

    Ok(Json(posts))
}

/// GET /posts/:slug — public read of a single published post.
async fn get_post_by_slug(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let post = sqlx::query_as::<_, Post>(
        r#"
        select * from posts
        where slug = $1 and status = 'published' and published_at <= now() and deleted_at is null
        "#,
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await
    .map_err(internal_err)?
    .ok_or((StatusCode::NOT_FOUND, "post not found".into()))?;

    Ok(Json(post))
}

/// GET /posts/preview/:token — lets an author share an unpublished draft
/// via a private link without the viewer needing to log in.
async fn get_post_by_preview_token(
    State(pool): State<PgPool>,
    Path(token): Path<Uuid>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let post = sqlx::query_as::<_, Post>("select * from posts where preview_token = $1")
        .bind(token)
        .fetch_optional(&pool)
        .await
        .map_err(internal_err)?
        .ok_or((StatusCode::NOT_FOUND, "post not found".into()))?;

    Ok(Json(post))
}

/// POST /posts — creates a draft. Requires auth.
async fn create_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(req): Json<CreatePostRequest>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let slug = db::slugify(&req.title);
    let html = db::render_markdown(&req.content_md);
    let reading_time = db::estimate_reading_minutes(&req.content_md);

    let post = sqlx::query_as::<_, Post>(
        r#"
        insert into posts (author_id, title, slug, excerpt, content_md, content_html, reading_time_min)
        values ($1, $2, $3, $4, $5, $6, $7)
        returning *
        "#,
    )
    .bind(user.id)
    .bind(&req.title)
    .bind(&slug)
    .bind(&req.excerpt)
    .bind(&req.content_md)
    .bind(&html)
    .bind(reading_time)
    .fetch_one(&pool)
    .await
    .map_err(internal_err)?;

    // Snapshot the first revision immediately.
    sqlx::query(
        "insert into post_revisions (post_id, title, content_md, created_by) values ($1, $2, $3, $4)",
    )
    .bind(post.id)
    .bind(&post.title)
    .bind(&post.content_md)
    .bind(user.id)
    .execute(&pool)
    .await
    .map_err(internal_err)?;

    Ok(Json(post))
}

/// PUT /posts/id/:id — edit a post (title/content/etc). Author-only, enforced
/// by Postgres RLS as a second line of defense even if this check is skipped.
async fn update_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let existing = sqlx::query_as::<_, Post>("select * from posts where id = $1 and author_id = $2")
        .bind(id)
        .bind(user.id)
        .fetch_optional(&pool)
        .await
        .map_err(internal_err)?
        .ok_or((StatusCode::NOT_FOUND, "post not found".into()))?;

    let title = req.title.unwrap_or(existing.title);
    let content_md = req.content_md.unwrap_or(existing.content_md);
    let excerpt = req.excerpt.or(existing.excerpt);
    let seo_description = req.seo_description.or(existing.seo_description);
    let og_image_url = req.og_image_url.or(existing.og_image_url);

    let html = db::render_markdown(&content_md);
    let reading_time = db::estimate_reading_minutes(&content_md);

    let post = sqlx::query_as::<_, Post>(
        r#"
        update posts set
          title = $1, content_md = $2, content_html = $3, excerpt = $4,
          seo_description = $5, og_image_url = $6, reading_time_min = $7
        where id = $8 and author_id = $9
        returning *
        "#,
    )
    .bind(&title)
    .bind(&content_md)
    .bind(&html)
    .bind(&excerpt)
    .bind(&seo_description)
    .bind(&og_image_url)
    .bind(reading_time)
    .bind(id)
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .map_err(internal_err)?;

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
    .await
    .map_err(internal_err)?;

    Ok(Json(post))
}

/// POST /posts/id/:id/publish — sets status + published_at.
/// Pass a future `published_at` to schedule instead of publishing now.
async fn publish_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<PublishPostRequest>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let when = req.published_at.unwrap_or_else(Utc::now);
    let status = if when > Utc::now() { "scheduled" } else { "published" };

    let post = sqlx::query_as::<_, Post>(
        r#"
        update posts set status = $1, published_at = $2
        where id = $3 and author_id = $4
        returning *
        "#,
    )
    .bind(status)
    .bind(when)
    .bind(id)
    .bind(user.id)
    .fetch_optional(&pool)
    .await
    .map_err(internal_err)?
    .ok_or((StatusCode::NOT_FOUND, "post not found".into()))?;

    Ok(Json(post))
}

/// DELETE /posts/id/:id — soft delete.
async fn delete_post(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("update posts set deleted_at = now() where id = $1 and author_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(internal_err)?;

    Ok(StatusCode::NO_CONTENT)
}

fn internal_err(e: sqlx::Error) -> (StatusCode, String) {
    tracing::error!("db error: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
}