use super::{AuthorStatus, PixStatus};

#[test]
fn serializes_author_status_as_camel_case() {
    let status = serde_json::to_value(AuthorStatus { is_author: true }).unwrap();

    assert_eq!(status, serde_json::json!({ "isAuthor": true }));
}

#[test]
fn serializes_pix_status_as_camel_case() {
    let status = serde_json::to_value(PixStatus { is_pix: true }).unwrap();

    assert_eq!(status, serde_json::json!({ "isPix": true }));
}
