use super::UploadUrls;

#[test]
fn serializes_upload_urls_as_camel_case() {
    let urls = UploadUrls {
        upload_url: "https://storage.example.test/upload".to_string(),
        public_url: "https://images.example.test/image.png".to_string(),
    };

    assert_eq!(
        serde_json::to_value(urls).unwrap(),
        serde_json::json!({
            "uploadUrl": "https://storage.example.test/upload",
            "publicUrl": "https://images.example.test/image.png",
        })
    );
}
