use aws_sdk_s3::error::SdkError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type RouteResult<T> = Result<T, RouteError>;

#[derive(Debug)]
pub enum RouteError {
    BadRequest(&'static str),
    Forbidden(&'static str),
    NotFound(&'static str),
    Database(sqlx::Error),
    S3(String),
}

impl RouteError {
    pub const fn bad_request(message: &'static str) -> Self {
        Self::BadRequest(message)
    }

    pub const fn not_found(message: &'static str) -> Self {
        Self::NotFound(message)
    }

    pub const fn forbidden(message: &'static str) -> Self {
        Self::Forbidden(message)
    }
}

impl<E> From<SdkError<E>> for RouteError {
    fn from(error: SdkError<E>) -> Self {
        Self::S3(error.to_string())
    }
}

impl From<sqlx::Error> for RouteError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, message).into_response(),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
            Self::Database(error) => {
                tracing::error!(%error, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
            }
            Self::S3(error) => {
                tracing::warn!("Error from S3: {error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
