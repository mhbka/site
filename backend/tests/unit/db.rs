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
