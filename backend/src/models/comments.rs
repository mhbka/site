use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub body: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentRequest {
    pub body: String,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::CreateCommentRequest;
    use uuid::Uuid;

    #[test]
    fn deserializes_parent_comment_id_from_camel_case() {
        let parent_comment_id = Uuid::nil();
        let request: CreateCommentRequest = serde_json::from_value(serde_json::json!({
            "body": "A reply",
            "parentCommentId": parent_comment_id,
        }))
        .unwrap();

        assert_eq!(request.body, "A reply");
        assert_eq!(request.parent_comment_id, Some(parent_comment_id));
    }
}
