use axum::{http::StatusCode, response::IntoResponse};

use super::RouteError;

#[test]
fn client_errors_keep_their_status_codes() {
    assert_eq!(
        RouteError::bad_request("invalid input")
            .into_response()
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        RouteError::forbidden("not an author")
            .into_response()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        RouteError::not_found("missing").into_response().status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn database_errors_become_internal_server_errors() {
    assert_eq!(
        RouteError::from(sqlx::Error::RowNotFound)
            .into_response()
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn object_storage_errors_become_internal_server_errors() {
    assert_eq!(
        RouteError::S3("storage unavailable".to_string())
            .into_response()
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
