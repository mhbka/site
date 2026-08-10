use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::jwk::JwkSet;
use sqlx::{Pool, Postgres};
use tokio::sync::{Mutex, MutexGuard};

const CACHE_VALIDITY: Duration = Duration::minutes(10);

/// State used by routes.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    jwkset: Arc<Mutex<CachedJwkSet>>
}

impl AppState {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            jwkset: Arc::new(Mutex::new(CachedJwkSet::new()))
        }
    }

    pub async fn jwkset(&self) -> MutexGuard<'_, CachedJwkSet> {
        self.jwkset
            .lock()
            .await
    }
}

#[derive(Debug, Clone)]
pub struct CachedJwkSet {
    set: JwkSet,
    last_fetched: DateTime<Utc>
}

impl CachedJwkSet {
    pub fn new() -> Self {

        Self {
            set: JwkSet { keys: vec![] },
            last_fetched: Utc::now() - Duration::weeks(1000) // immediately outdated, so we invalidate the default set
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
        Utc::now() > self.last_fetched + CACHE_VALIDITY
    }
}