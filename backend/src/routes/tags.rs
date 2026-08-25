use axum::{extract::State, routing::get, Json, Router};

use crate::{models::tags::TagSummary, routes::error::RouteResult, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_tags))
}

/// GET /tags — tags and their public post counts, most used first.
async fn list_tags(State(app_state): State<AppState>) -> RouteResult<Json<Vec<TagSummary>>> {
    let tags = sqlx::query_as::<_, TagSummary>(
        r#"
        select tag, count(*)::bigint as count
        from posts cross join lateral unnest(tags) as tag
        where status = 'published' and published_at <= now() and deleted_at is null
        group by tag
        order by count desc, tag asc
        "#,
    )
    .fetch_all(&app_state.pool)
    .await?;

    Ok(Json(tags))
}
