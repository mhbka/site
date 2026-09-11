use aws_sdk_s3::error::SdkError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Standard result type returned by API handlers.
pub type RouteResult<T> = Result<T, RouteError>;

/// Represents a client-safe error produced by a route handler.
#[derive(Debug)]
pub enum RouteError {
    BadRequest(&'static str),
    Forbidden(&'static str),
    NotFound(&'static str),
    Database(sqlx::Error),
    S3(String),
}

impl RouteError {
    /// Creates a bad-request response error.
    pub const fn bad_request(message: &'static str) -> Self {
        Self::BadRequest(message)
    }

    /// Creates a not-found response error.
    pub const fn not_found(message: &'static str) -> Self {
        Self::NotFound(message)
    }

    /// Creates a forbidden response error.
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
            Self::BadRequest(message) => {
                tracing::info!(%message, "request rejected");
                (StatusCode::BAD_REQUEST, message).into_response()
            }
            Self::Forbidden(message) => {
                tracing::info!(%message, "request forbidden");
                (StatusCode::FORBIDDEN, message).into_response()
            }
            Self::NotFound(message) => {
                tracing::info!(%message, "requested resource was not found");
                (StatusCode::NOT_FOUND, message).into_response()
            }
            Self::Database(error) => {
                tracing::error!(%error, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
            }
            Self::S3(error) => {
                tracing::error!(%error, "object storage error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/routes/error.rs"]
mod tests;
