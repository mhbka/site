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
