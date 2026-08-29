mod auth;
mod db;
mod models;
mod routes;
mod s3;
mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let s3_account_id = std::env::var("S3_ACCOUNT_ID").expect("S3_ACCOUNT_ID is in env");
    let s3_access_key_id = std::env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID is in env");
    let s3_access_key_secret =
        std::env::var("S3_ACCESS_KEY_SECRET").expect("S3_ACCESS_KEY_SECRET is in env");
    let s3_blogpost_bucket_name = std::env::var("S3_BLOGPOST_MEDIA_BUCKET_NAME")
        .expect("S3_BLOGPOST_MEDIA_BUCKET_NAME is in env");
    let s3_blogpost_bucket_url = std::env::var("S3_BLOGPOST_MEDIA_BUCKET_URL")
        .expect("S3_BLOGPOST_MEDIA_BUCKET_URL is in env");

    let app_state = AppState::new(
        db::connect().await?,
        s3_account_id,
        s3_access_key_id,
        s3_access_key_secret,
        s3_blogpost_bucket_name,
        s3_blogpost_bucket_url,
    )
    .await;

    // Loosen this to actual frontend origin before going to production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/posts", routes::posts::router())
        .nest("/tags", routes::tags::router())
        .nest("/users", routes::users::router())
        .nest("/comments", routes::comments::router())
        .nest("/media", routes::media::router())
        .with_state(app_state)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("listening on :{port}");
    axum::serve(listener, app).await?;

    Ok(())
}
