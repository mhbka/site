mod auth;
mod db;
mod models;
mod routes;
mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::state::{AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let app_state = AppState::new(db::connect().await?);

    // Loosen this to actual frontend origin before going to production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/posts", routes::posts::router())
        .nest("/users", routes::users::router())
        .nest("/comments", routes::comments::router())
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
