use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use crate::dtos::file::UploadFileRequest;

#[derive(Serialize)]
pub struct FileCollection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub user_id: ObjectId,

    pub name: String,
    pub size: u64, // bytes
    pub cid: String,
    pub is_expired: bool,
    pub mime_type: String,

    pub uploaded_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,

    pub max_downloads: u8,
    pub download_count: u64,
    pub download_history: Vec<DownloadEntry>,
}

#[derive(Debug, Serialize)]
pub struct DownloadEntry {
    pub downloaded_at: DateTime<Utc>,
    pub ip: String,
    pub user_agent: String,
}

impl From<UploadFileRequest> for FileCollection {
    fn from(payload: UploadFileRequest) -> Self {
        FileCollection {
            id: None,
            user_id: payload.user_id,
            name: payload.file_name,
            size: payload.size,
            cid: payload.cid,
            is_expired: payload.is_expired,
            mime_type: payload.mime_type,
            uploaded_at: Utc::now(),
            expires_at: payload.expires_at,
            max_downloads: payload.max_downloads,
            download_count: 0,
            download_history: Vec::new(),
        }
    }
}
