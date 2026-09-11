mod auth;
mod db;
mod models;
mod routes;
mod s3;
mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

#[tokio::main]
/// Starts the HTTP server with its shared state and middleware.
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .init();

    let s3_account_id = std::env::var("S3_ACCOUNT_ID").expect("S3_ACCOUNT_ID is in env");
    let s3_access_key_id = std::env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID is in env");
    let s3_access_key_secret =
        std::env::var("S3_ACCESS_KEY_SECRET").expect("S3_ACCESS_KEY_SECRET is in env");
    let s3_blogpost_bucket_name = std::env::var("S3_BLOGPOST_MEDIA_BUCKET_NAME")
        .expect("S3_BLOGPOST_MEDIA_BUCKET_NAME is in env");
    let s3_blogpost_bucket_url = std::env::var("S3_BLOGPOST_MEDIA_BUCKET_URL")
        .expect("S3_BLOGPOST_MEDIA_BUCKET_URL is in env");
    let s3_pix_bucket_name =
        std::env::var("S3_PIX_BUCKET_NAME").expect("S3_PIX_BUCKET_NAME is in env");
    let s3_pix_bucket_url =
        std::env::var("S3_PIX_BUCKET_URL").expect("S3_PIX_BUCKET_URL is in env");

    let app_state = AppState::new(
        db::connect().await?,
        s3_account_id,
        s3_access_key_id,
        s3_access_key_secret,
        s3_blogpost_bucket_name,
        s3_blogpost_bucket_url,
        s3_pix_bucket_name,
        s3_pix_bucket_url,
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
        .nest("/pix", routes::pix::router())
        .with_state(app_state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let address = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(%address, "backend started");
    axum::serve(listener, app).await?;

    Ok(())
}
