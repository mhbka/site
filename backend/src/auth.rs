use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

/// How long a cached JWK set is valid before we should query for one again.
const JWK_SET_CACHE_VALIDITY: Duration = Duration::minutes(10);

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
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Parse the auth header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|err| {
                tracing::debug!("Failed to extract bearer token: {err}");
                AuthError(StatusCode::UNAUTHORIZED, "missing bearer token")
            })?;

        // Update the cached JWK set
        let mut jwkset = state.jwkset().await;
        if let Err(err) = verify_replace_jwk_set(&mut jwkset).await {
            tracing::warn!("Failed to verify and replace JWK set: {err}");
            return Err(AuthError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while authenticating",
            ));
        }

        // Find the valid JWK from the set
        let header = decode_header(bearer.token())
            .map_err(|_| AuthError(StatusCode::UNAUTHORIZED, "JWT couldn't be decoded"))?;
        let kid = header.kid.ok_or(AuthError(
            StatusCode::UNAUTHORIZED,
            "JWT doesn't contain any kid header",
        ))?;
        let jwk = jwkset.jwks().find(&kid).ok_or(AuthError(
            StatusCode::UNAUTHORIZED,
            "JWT doesn't contain a valid kid header",
        ))?;
        let jwk_key = DecodingKey::from_jwk(jwk).map_err(|err| {
            tracing::warn!("Failed to create decoding key from the JWK: {err}");
            AuthError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while authenticating",
            )
        })?;

        // Set expected audience and algorithm
        let mut validation = Validation::default();
        validation.set_audience(&["authenticated"]);
        validation.algorithms = vec![Algorithm::ES256];

        let token_data =
            decode::<SupabaseClaims>(bearer.token(), &jwk_key, &validation).map_err(|err| {
                tracing::debug!("Failed to validate token: {err}");
                AuthError(StatusCode::UNAUTHORIZED, "Invalid or expired JWT")
            })?;

        Ok(AuthUser {
            id: token_data.claims.sub,
        })
    }
}

/// Optional variant for routes that behave differently for logged-in vs.
/// anonymous users (e.g. showing your own drafts in a public listing).
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[axum::async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}

/// Verify the `CachedJwkSet` is not outdated.
///
/// If it is, fetch and replace it; return an error if it fails at any point.
async fn verify_replace_jwk_set(current: &mut CachedJwkSet) -> anyhow::Result<()> {
    if current.outdated() {
        let jwkset_url = std::env::var("JWKSET_URL")?;
        let new_set = reqwest::get(jwkset_url)
            .await?
            .error_for_status()?
            .json::<JwkSet>()
            .await?;
        current.update(new_set);
    }

    Ok(())
}

/// Abstraction for caching JWK sets.
#[derive(Debug, Clone)]
pub struct CachedJwkSet {
    set: JwkSet,
    last_fetched: DateTime<Utc>,
}

impl CachedJwkSet {
    pub fn new() -> Self {
        Self {
            set: JwkSet { keys: vec![] },
            last_fetched: Utc::now() - JWK_SET_CACHE_VALIDITY - Duration::weeks(1000), // immediately outdated, so we invalidate the default set
        }
    }

    pub fn update(&mut self, set: JwkSet) {
        self.set = set;
        self.last_fetched = Utc::now();
    }

    pub fn jwks(&self) -> &JwkSet {
        &self.set
    }

    pub fn outdated(&self) -> bool {
        Utc::now() > self.last_fetched + JWK_SET_CACHE_VALIDITY
    }
}
