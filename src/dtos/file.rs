use chrono::{DateTime, Utc};
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct UploadFileRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub file_name: String,

    #[validate(custom(function = "validate_expires_at"))]
    pub expires_at: DateTime<Utc>,

    #[validate(range(exclusive_min = 0, max = 10))]
    pub max_downloads: u128,
}

fn validate_expires_at(date: &DateTime<Utc>) -> Result<(), ValidationError> {
    if *date <= Utc::now() {
        return Err(ValidationError::new("`expires_at` must_be_in_future"));
    }
    Ok(())
}
