use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect() -> anyhow::Result<PgPool> {
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    Ok(pool)
}

/// Render Markdown -> sanitized-ish HTML. `comrak` covers GFM (tables,
/// strikethrough, autolinks, task lists). Run this once on save, not per
/// page view, and store the result in `content_html`.
pub fn render_markdown(md: &str) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.render.unsafe_ = false; // strips raw HTML/script from user input
    comrak::markdown_to_html(md, &options)
}

/// Slugify a title, e.g. "Hello, World!" -> "hello-world".
/// Call this once at creation time; keep the slug stable after that even
/// if the title changes, so old links don't break.
pub fn slugify(title: &str) -> String {
    slug::slugify(title)
}

/// Rough reading time estimate at ~200 words per minute.
pub fn estimate_reading_minutes(md: &str) -> i32 {
    let words = md.split_whitespace().count();
    ((words as f32 / 200.0).ceil() as i32).max(1)
}