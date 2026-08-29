use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use crate::{auth::CachedJwkSet, s3::S3};

/// State used by routes.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub s3: S3,
    jwkset: Arc<Mutex<CachedJwkSet>>,
}

impl AppState {
    pub async fn new(
        pool: Pool<Postgres>,
        s3_account_id: String,
        s3_access_key_id: String,
        s3_access_key_secret: String,
        s3_blogpost_bucket_name: String,
        s3_blogpost_bucket_url: String,
    ) -> Self {
        let s3 = S3::new(
            s3_account_id,
            s3_access_key_id,
            s3_access_key_secret,
            s3_blogpost_bucket_name,
            s3_blogpost_bucket_url,
        )
        .await;
        Self {
            pool,
            s3,
            jwkset: Arc::new(Mutex::new(CachedJwkSet::new())),
        }
    }

    pub async fn jwkset(&self) -> MutexGuard<'_, CachedJwkSet> {
        self.jwkset.lock().await
    }
}
