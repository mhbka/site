use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

/// Claims we care about from the Supabase-issued JWT.
/// Supabase includes plenty more (aud, role, session_id, etc.) — add as needed.
#[derive(Debug, Deserialize)]
struct SupabaseClaims {
    sub: Uuid, // this is the user's auth.uid()
    exp: usize,
}

/// Represents an authenticated user.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
}

pub struct AuthError(pub StatusCode, pub &'static str);

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError(StatusCode::UNAUTHORIZED, "missing bearer token"))?;

        let secret = std::env::var("SUPABASE_JWT_SECRET")
            .map_err(|_| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "server misconfigured"))?;

        let mut validation = Validation::default();

        // Supabase's default audience for its JWTs; adjust if you customized it.
        validation.set_audience(&["authenticated"]);

        let token_data = decode::<SupabaseClaims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map_err(|_| AuthError(StatusCode::UNAUTHORIZED, "invalid or expired token"))?;

        Ok(AuthUser {
            id: token_data.claims.sub,
        })
    }
}

/// Optional variant for routes that behave differently for logged-in vs.
/// anonymous users (e.g. showing your own drafts in a public listing).
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
