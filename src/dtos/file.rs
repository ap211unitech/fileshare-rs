use axum::body::Bytes;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Validate)]
pub struct UploadFileRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub file_name: String,

    pub file_data: Bytes,
    pub size: u128, // bytes
    pub cid: String,
    pub is_expired: bool,
    pub mime_type: String,
    pub hashed_password: Option<String>,

    #[validate(custom(function = "validate_expires_at"))]
    pub expires_at: DateTime<Utc>,

    #[validate(range(exclusive_min = 0, max = 10, message = "expected between 1 to 10"))]
    pub max_downloads: u128,
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
            file_name: Default::default(),
            expires_at: Default::default(),
            max_downloads: Default::default(),
            size: 0,
            cid: format!("cid"),
            is_expired: false,
            mime_type: format!("mime_type"),
            hashed_password: None,
            file_data: Bytes::new(),
        }
    }
}
