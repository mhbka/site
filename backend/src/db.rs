use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect() -> anyhow::Result<PgPool> {
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    Ok(pool)
}

/// Slugify a title, e.g. "Hello, World!" -> "hello-world".
/// Call this once at creation time; keep the slug stable after that even
/// if the title changes, so old links don't break.
pub fn slugify(title: &str) -> String {
    slug::slugify(title)
}
