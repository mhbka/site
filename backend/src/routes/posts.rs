use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
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

async fn require_author(pool: &PgPool, user_id: Uuid) -> RouteResult<()> {
    let is_author = sqlx::query_scalar::<_, bool>(
        "select exists(select 1 from profiles where user_id = $1 and is_author)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if is_author {
        Ok(())
    } else {
        Err(RouteError::forbidden("author access required"))
    }
}

/// GET /posts — published posts only, newest first. Paginate as needed.
const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Deserialize)]
struct ListPostsQuery {
    page: Option<u32>,
    size: Option<u32>,
}

impl ListPostsQuery {
    fn pagination(self) -> Result<(i64, i64), RouteError> {
        let page = self.page.unwrap_or(1);
        let size = self.size.unwrap_or(DEFAULT_PAGE_SIZE);

        if page == 0 {
            return Err(RouteError::bad_request("page must be at least 1"));
        }
        if size == 0 || size > MAX_PAGE_SIZE {
            return Err(RouteError::bad_request("size must be between 1 and 100"));
        }

        let offset = u64::from(page - 1) * u64::from(size);
        Ok((i64::from(size), offset as i64))
    }
}

async fn list_posts(
    State(pool): State<PgPool>,
    OptionalAuthUser(_user): OptionalAuthUser,
    Query(query): Query<ListPostsQuery>,
) -> RouteResult<Json<Vec<PostSummary>>> {
    let (size, offset) = query.pagination()?;
    let posts = sqlx::query_as::<_, PostSummary>(
        r#"
        select id, title, slug, thumbnail_url, published_at
        from posts
        where status = 'published' and published_at <= now() and deleted_at is null
        order by published_at desc
        limit $1 offset $2
        "#,
    )
    .bind(size)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    Ok(Json(posts))
}

#[cfg(test)]
mod tests {
    use super::{ListPostsQuery, DEFAULT_PAGE_SIZE};

    #[test]
    fn uses_default_pagination_values() {
        assert_eq!(
            ListPostsQuery {
                page: None,
                size: None
            }
            .pagination()
            .unwrap(),
            (i64::from(DEFAULT_PAGE_SIZE), 0)
        );
    }

    #[test]
    fn calculates_offset_for_requested_page() {
        assert_eq!(
            ListPostsQuery {
                page: Some(3),
                size: Some(20)
            }
            .pagination()
            .unwrap(),
            (20, 40)
        );
    }

    #[test]
    fn rejects_invalid_pagination_values() {
        assert!(ListPostsQuery {
            page: Some(0),
            size: Some(20)
        }
        .pagination()
        .is_err());
        assert!(ListPostsQuery {
            page: Some(1),
            size: Some(101)
        }
        .pagination()
        .is_err());
    }
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
    require_author(&pool, user.id).await?;

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
    require_author(&pool, user.id).await?;

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
    require_author(&pool, user.id).await?;

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
    require_author(&pool, user.id).await?;

    sqlx::query("update posts set deleted_at = now() where id = $1 and author_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
