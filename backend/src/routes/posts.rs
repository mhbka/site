use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::models::posts::*;
use crate::routes::error::{RouteError, RouteResult};
use crate::{
    auth::{AuthUser, OptionalAuthUser},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/drafts", get(list_drafts))
        .route("/:slug", get(get_post_by_slug))
        .route(
            "/id/:id",
            get(get_post_by_id).put(update_post).delete(delete_post),
        )
        .route("/id/:id/publish", post(publish_post))
        .route("/id/:id/draft", post(move_post_to_draft))
}

async fn require_author(pool: &PgPool, user_id: Uuid) -> RouteResult<()> {
    if crate::routes::users::is_author(pool, user_id).await? {
        Ok(())
    } else {
        Err(RouteError::forbidden("author access required"))
    }
}

/// GET  — published posts only, newest first. Paginate as needed.
const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Deserialize)]
struct ListPostsQuery {
    page: Option<u32>,
    size: Option<u32>,
    tag: Option<String>,
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

    fn tag(&self) -> Option<String> {
        self.tag
            .as_ref()
            .map(|tag| normalize_tag(tag))
            .filter(|tag| !tag.is_empty())
    }
}

async fn list_posts(
    State(app_state): State<AppState>,
    OptionalAuthUser(_user): OptionalAuthUser,
    Query(query): Query<ListPostsQuery>,
) -> RouteResult<Json<Vec<PostSummary>>> {
    let tag = query.tag();
    let (size, offset) = query.pagination()?;
    let posts = sqlx::query_as::<_, PostSummary>(
        r#"
        select id, title, slug, thumbnail_url, tags, published_at
        from posts
        where status = 'published' and published_at <= now() and deleted_at is null
          and ($1::text is null or tags @> array[$1::text])
        order by published_at desc
        limit $2 offset $3
        "#,
    )
    .bind(tag)
    .bind(size)
    .bind(offset)
    .fetch_all(&app_state.pool)
    .await?;

    Ok(Json(posts))
}

/// GET /drafts — the authenticated author's drafts, newest updated first.
async fn list_drafts(
    State(app_state): State<AppState>,
    user: AuthUser,
) -> RouteResult<Json<Vec<DraftPostSummary>>> {
    require_author(&app_state.pool, user.id).await?;

    let posts = sqlx::query_as::<_, DraftPostSummary>(
        r#"
        select id, title, slug, thumbnail_url, tags, updated_at
        from posts
        where status = 'draft' and author_id = $1 and deleted_at is null
        order by updated_at desc
        "#,
    )
    .bind(user.id)
    .fetch_all(&app_state.pool)
    .await?;

    Ok(Json(posts))
}

#[cfg(test)]
mod tests {
    use super::{normalize_tags, ListPostsQuery, DEFAULT_PAGE_SIZE};

    #[test]
    fn uses_default_pagination_values() {
        assert_eq!(
            ListPostsQuery {
                page: None,
                size: None,
                tag: None,
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
                size: Some(20),
                tag: None,
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
            size: Some(20),
            tag: None,
        }
        .pagination()
        .is_err());
        assert!(ListPostsQuery {
            page: Some(1),
            size: Some(101),
            tag: None,
        }
        .pagination()
        .is_err());
    }

    #[test]
    fn normalizes_and_deduplicates_tags() {
        assert_eq!(
            normalize_tags(vec![
                "Astro".to_string(),
                "Java Script".to_string(),
                "astro".to_string(),
                " ".to_string(),
            ]),
            vec!["astro", "javascript"],
        );
    }

    #[test]
    fn normalizes_the_tag_filter() {
        assert_eq!(
            ListPostsQuery {
                page: None,
                size: None,
                tag: Some("Java Script".to_string())
            }
            .tag(),
            Some("javascript".to_string())
        );
    }
}

/// GET /:slug — public read of a single published post.
async fn get_post_by_slug(
    State(app_state): State<AppState>,
    Path(slug): Path<String>,
) -> RouteResult<Json<Post>> {
    let post = sqlx::query_as::<_, Post>(
        r#"
        select * from posts
        where slug = $1 and status = 'published' and published_at <= now() and deleted_at is null
        "#,
    )
    .bind(slug)
    .fetch_optional(&app_state.pool)
    .await?
    .ok_or(RouteError::not_found("post not found"))?;

    Ok(Json(post))
}

/// GET /id/:id — author-only access to a post, including drafts.
async fn get_post_by_id(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<Json<Post>> {
    require_author(&app_state.pool, user.id).await?;

    let post = sqlx::query_as::<_, Post>(
        "select * from posts where id = $1 and author_id = $2 and deleted_at is null",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&app_state.pool)
    .await?
    .ok_or(RouteError::not_found("post not found"))?;

    Ok(Json(post))
}

/// POST  — creates a draft. Requires auth.
async fn create_post(
    State(app_state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreatePostRequest>,
) -> RouteResult<Json<Post>> {
    require_author(&app_state.pool, user.id).await?;

    let slug = requested_slug(req.slug, &req.title)?;
    let tags = normalize_tags(req.tags.unwrap_or_default());
    let post = sqlx::query_as::<_, Post>(
        r#"
        insert into posts (author_id, title, slug, content_md, tags)
        values ($1, $2, $3, $4, $5)
        returning *
        "#,
    )
    .bind(user.id)
    .bind(&req.title)
    .bind(&slug)
    .bind(&req.content_md)
    .bind(&tags)
    .fetch_one(&app_state.pool)
    .await?;

    // Snapshot the first revision immediately.
    sqlx::query(
        "insert into post_revisions (post_id, title, content_md, created_by) values ($1, $2, $3, $4)",
    )
    .bind(post.id)
    .bind(&post.title)
    .bind(&post.content_md)
    .bind(user.id)
    .execute(&app_state.pool)
    .await?;

    Ok(Json(post))
}

/// PUT /id/:id — edit a post (title/content/etc). Author-only.
async fn update_post(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> RouteResult<Json<Post>> {
    require_author(&app_state.pool, user.id).await?;

    let existing =
        sqlx::query_as::<_, Post>("select * from posts where id = $1 and author_id = $2")
            .bind(id)
            .bind(user.id)
            .fetch_optional(&app_state.pool)
            .await?
            .ok_or(RouteError::not_found("post not found"))?;

    let title = req.title.unwrap_or(existing.title);
    let content_md = req.content_md.unwrap_or(existing.content_md);
    let thumbnail_url = req.thumbnail_url.or(existing.thumbnail_url);
    let slug = match req.slug {
        Some(slug) => requested_slug(Some(slug), &title)?,
        None => existing.slug,
    };
    let tags = req.tags.map(normalize_tags).unwrap_or(existing.tags);

    let post = sqlx::query_as::<_, Post>(
        r#"
        update posts set
          title = $1, content_md = $2, thumbnail_url = $3, slug = $4, tags = $5
        where id = $6 and author_id = $7
        returning *
        "#,
    )
    .bind(&title)
    .bind(&content_md)
    .bind(&thumbnail_url)
    .bind(&slug)
    .bind(&tags)
    .bind(id)
    .bind(user.id)
    .fetch_one(&app_state.pool)
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
    .execute(&app_state.pool)
    .await?;

    Ok(Json(post))
}

fn requested_slug(slug: Option<String>, title: &str) -> Result<String, RouteError> {
    match slug {
        Some(slug) if !slug.is_empty() && !db::is_valid_slug(&slug) => Err(
            RouteError::bad_request("slug may contain only letters, numbers, and hyphens"),
        ),
        Some(slug) if !slug.is_empty() => Ok(slug),
        _ => Ok(db::slugify(title)),
    }
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();

    for tag in tags {
        let tag = normalize_tag(&tag);
        if !tag.is_empty() && !normalized.contains(&tag) {
            normalized.push(tag);
        }
    }

    normalized
}

fn normalize_tag(tag: &str) -> String {
    tag.to_lowercase().split_whitespace().collect()
}

/// POST /id/:id/publish — publishes immediately.
async fn publish_post(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<Json<Post>> {
    require_author(&app_state.pool, user.id).await?;

    let post = sqlx::query_as::<_, Post>(
        r#"
        update posts set status = $1, published_at = now()
        where id = $2 and author_id = $3 and status = 'draft' and deleted_at is null
        returning *
        "#,
    )
    .bind(PostStatus::Published)
    .bind(id)
    .bind(user.id)
    .fetch_optional(&app_state.pool)
    .await?
    .ok_or(RouteError::not_found("post not found"))?;

    Ok(Json(post))
}

/// POST /id/:id/draft — removes a published post from the public blog.
async fn move_post_to_draft(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<Json<Post>> {
    require_author(&app_state.pool, user.id).await?;

    let post = sqlx::query_as::<_, Post>(
        r#"
        update posts set status = $1, published_at = null
        where id = $2 and author_id = $3 and status = 'published' and deleted_at is null
        returning *
        "#,
    )
    .bind(PostStatus::Draft)
    .bind(id)
    .bind(user.id)
    .fetch_optional(&app_state.pool)
    .await?
    .ok_or(RouteError::not_found("post not found"))?;

    Ok(Json(post))
}

/// DELETE /id/:id — soft delete.
async fn delete_post(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> RouteResult<StatusCode> {
    require_author(&app_state.pool, user.id).await?;

    sqlx::query("update posts set deleted_at = now() where id = $1 and author_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&app_state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
