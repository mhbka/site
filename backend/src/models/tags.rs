use serde::Serialize;

/// A tag currently used by one or more public posts.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TagSummary {
    pub tag: String,
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::TagSummary;

    #[test]
    fn serializes_tag_summary_as_camel_case() {
        let value = serde_json::to_value(TagSummary {
            tag: "rust".to_string(),
            count: 3,
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "tag": "rust", "count": 3 }));
    }
}
