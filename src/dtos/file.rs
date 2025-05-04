use axum::body::Bytes;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::models::file::FileCollection;

#[derive(Debug, Clone, Validate)]
pub struct UploadFileRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub file_name: String,

    pub user_id: ObjectId,

    pub file_data: Bytes,

    #[validate(range(
        exclusive_min = 0,
        max = 10_000_000,
        message = "size should be less than 10 MB"
    ))]
    pub size: u64, // bytes
    pub cid: String,
    pub mime_type: String,
    pub password: String,

    #[validate(custom(function = "validate_expires_at"))]
    pub expires_at: DateTime<Utc>,

    #[validate(range(exclusive_min = 0, max = 10, message = "expected between 1 to 10"))]
    pub max_downloads: u8,
}

fn validate_expires_at(date: &DateTime<Utc>) -> Result<(), ValidationError> {
    if *date <= Utc::now() {
        return Err(ValidationError::new("`expires_at` must_be_in_future"));
    }
    Ok(())
}

impl Default for UploadFileRequest {
    fn default() -> Self {
        Self {
            user_id: ObjectId::new(),
            file_name: Default::default(),
            expires_at: Default::default(),
            max_downloads: Default::default(),
            size: 0,
            cid: format!("cid"),
            mime_type: format!("mime_type"),
            password: "default-password".to_string(),
            file_data: Bytes::new(),
        }
    }
}

#[derive(Serialize)]
pub struct UploadFileResponse {
    pub id: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct DownloadFileRequest {
    pub file_id: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct UserFilesResponse {
    pub files: Vec<FileCollection>,
}
