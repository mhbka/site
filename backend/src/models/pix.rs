use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Pix {
    pub id: Uuid,
    pub public_url: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PixPage {
    pub images: Vec<Pix>,
    pub next_before: Option<Uuid>,
    pub has_more: bool,
}
