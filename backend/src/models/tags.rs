use serde::Serialize;

/// A tag currently used by one or more public posts.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TagSummary {
    pub tag: String,
    pub count: i64,
}

#[cfg(test)]
#[path = "../../tests/unit/models/tags.rs"]
mod tests;
