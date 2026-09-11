use sqlx::{postgres::PgPoolOptions, PgPool};

/// Opens the application's PostgreSQL connection pool.
pub async fn connect() -> anyhow::Result<PgPool> {
    tracing::info!("connecting to database");
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    tracing::info!(max_connections = 10, "database connection pool established");
    Ok(pool)
}

/// Slugify a title, e.g. "Hello, World!" -> "hello-world".
/// Call this once at creation time; keep the slug stable after that even
/// if the title changes, so old links don't break.
pub fn slugify(title: &str) -> String {
    slug::slugify(title)
}

/// A custom slug may contain letters, numbers, and hyphens only.
pub fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || character == b'-')
}

#[cfg(test)]
#[path = "../tests/unit/db.rs"]
mod tests;
