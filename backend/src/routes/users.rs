use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::routes::error::RouteResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/is-author", get(is_author_status))
}

pub async fn is_author(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "select exists(select 1 from profiles where user_id = $1 and is_author)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorStatus {
    is_author: bool,
}

/// GET /users/is-author — returns whether the authenticated user can author posts.
async fn is_author_status(
    State(app_state): State<AppState>,
    user: AuthUser,
) -> RouteResult<Json<AuthorStatus>> {
    tracing::info!("USER: {}", user.id);
    Ok(Json(AuthorStatus {
        is_author: is_author(&app_state.pool, user.id).await?,
    }))
}

#[cfg(test)]
mod tests {
    use super::AuthorStatus;

    #[test]
    fn serializes_author_status_as_camel_case() {
        let status = serde_json::to_value(AuthorStatus { is_author: true }).unwrap();

        assert_eq!(status, serde_json::json!({ "isAuthor": true }));
    }
}
