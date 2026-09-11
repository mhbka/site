use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::routes::error::RouteResult;
use crate::state::AppState;

/// Builds the authenticated-user status API routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/is-author", get(is_author_status))
        .route("/is-pix", get(is_pix_status))
}

/// Checks whether a user has Pix gallery access.
pub async fn is_pix(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "select exists(select 1 from profiles where user_id = $1 and is_pix)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Checks whether a user has blog-author access.
pub async fn is_author(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "select exists(select 1 from profiles where user_id = $1 and is_author)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Returns the authenticated user's author permission.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorStatus {
    is_author: bool,
}

/// Returns the authenticated user's Pix permission.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PixStatus {
    is_pix: bool,
}

/// GET /users/is-author — returns whether the authenticated user can author posts.
async fn is_author_status(
    State(app_state): State<AppState>,
    user: AuthUser,
) -> RouteResult<Json<AuthorStatus>> {
    let is_author = is_author(&app_state.pool, user.id).await?;
    tracing::info!(user_id = %user.id, is_author, "author permission checked");
    Ok(Json(AuthorStatus { is_author }))
}

/// Returns whether the authenticated user can upload to Pix.
async fn is_pix_status(
    State(app_state): State<AppState>,
    user: AuthUser,
) -> RouteResult<Json<PixStatus>> {
    let is_pix = is_pix(&app_state.pool, user.id).await?;
    tracing::info!(user_id = %user.id, is_pix, "Pix permission checked");
    Ok(Json(PixStatus { is_pix }))
}

#[cfg(test)]
#[path = "../../tests/unit/routes/users.rs"]
mod tests;
