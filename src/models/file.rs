use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

#[derive(Serialize)]
pub struct FileCollection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub user_id: ObjectId,

    pub name: String,
    pub size: u128, // bytes
    pub path: String,
    pub is_expired: bool,
    pub mime_type: String,
    pub hashed_password: Option<String>,

    pub uploaded_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,

    pub max_downloads: u128,
    pub download_count: u32,
    pub download_history: Vec<DownloadEntry>,
}

#[derive(Debug, Serialize)]
pub struct DownloadEntry {
    pub downloaded_at: DateTime<Utc>,
    pub ip: String,
    pub user_agent: String,
}
