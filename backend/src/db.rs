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

/// A custom slug may contain letters, numbers, and hyphens only.
pub fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || character == b'-')
}

#[cfg(test)]
mod tests {
    use super::is_valid_slug;

    #[test]
    fn accepts_slugs_with_letters_numbers_and_hyphens() {
        assert!(is_valid_slug("my-post-2026"));
        assert!(is_valid_slug("Post42"));
    }

    #[test]
    fn rejects_empty_or_invalid_custom_slugs() {
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("my post"));
        assert!(!is_valid_slug("my_post"));
        assert!(!is_valid_slug("my/post"));
    }
}
