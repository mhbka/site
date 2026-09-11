use super::{normalize_tags, ListPostsQuery, DEFAULT_PAGE_SIZE};

#[test]
fn uses_default_pagination_values() {
    assert_eq!(
        ListPostsQuery {
            page: None,
            size: None,
            tag: None,
        }
        .pagination()
        .unwrap(),
        (i64::from(DEFAULT_PAGE_SIZE), 0)
    );
}

#[test]
fn calculates_offset_for_requested_page() {
    assert_eq!(
        ListPostsQuery {
            page: Some(3),
            size: Some(20),
            tag: None,
        }
        .pagination()
        .unwrap(),
        (20, 40)
    );
}

#[test]
fn rejects_invalid_pagination_values() {
    assert!(ListPostsQuery {
        page: Some(0),
        size: Some(20),
        tag: None,
    }
    .pagination()
    .is_err());
    assert!(ListPostsQuery {
        page: Some(1),
        size: Some(101),
        tag: None,
    }
    .pagination()
    .is_err());
}

#[test]
fn normalizes_and_deduplicates_tags() {
    assert_eq!(
        normalize_tags(vec![
            "Astro".to_string(),
            "Java Script".to_string(),
            "astro".to_string(),
            " ".to_string(),
        ]),
        vec!["astro", "javascript"],
    );
}

#[test]
fn normalizes_the_tag_filter() {
    assert_eq!(
        ListPostsQuery {
            page: None,
            size: None,
            tag: Some("Java Script".to_string())
        }
        .tag(),
        Some("javascript".to_string())
    );
}
