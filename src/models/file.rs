use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::dtos::file::UploadFileRequest;

#[derive(Serialize, Deserialize)]
pub struct FileCollection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub user_id: ObjectId,

    pub name: String,
    pub size: u64, // bytes
    pub cid: String,
    pub mime_type: String,

    pub uploaded_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,

    pub max_downloads: u8,
    pub download_count: u8,
    pub downloads: Vec<DownloadEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadEntry {
    ip_address: String,
    user_agent: Option<String>,
    downloaded_at: DateTime<Utc>,
}

impl DownloadEntry {
    pub fn new(ip_address: String, user_agent: Option<String>) -> Self {
        DownloadEntry {
            ip_address,
            user_agent,
            downloaded_at: Utc::now(),
        }
    }
}

impl From<UploadFileRequest> for FileCollection {
    fn from(payload: UploadFileRequest) -> Self {
        FileCollection {
            id: None,
            user_id: payload.user_id,
            name: payload.file_name,
            size: payload.size,
            cid: payload.cid,
            mime_type: payload.mime_type,
            uploaded_at: Utc::now(),
            expires_at: payload.expires_at,
            max_downloads: payload.max_downloads,
            download_count: 0,
            downloads: vec![],
        }
    }
}
