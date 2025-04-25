use axum::{extract::Multipart, response::IntoResponse};
use chrono::DateTime;
use validator::Validate;

use crate::{dtos::file::UploadFileRequest, error::AppError, utils::extractor::ExtractAuthAgent};

pub async fn upload_file(
    agent: ExtractAuthAgent,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut upload_file_request = UploadFileRequest::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        let form_key = field
            .name()
            .map(str::to_string)
            .ok_or_else(|| AppError::Internal("Error reading field name".to_string()))?;

        match form_key.as_str() {
            "expires_at" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(format!("Error reading text: {}", e)))?;

                upload_file_request.expires_at = text
                    .parse::<DateTime<chrono::Utc>>()
                    .map_err(|e| AppError::Internal(format!("Error parsing datetime: {}", e)))?;
            }
            "max_downloads" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(format!("Error reading text: {}", e)))?;

                upload_file_request.max_downloads = text.parse::<u128>().map_err(|e| {
                    AppError::Internal(format!("Error parsing max_downloads: {}", e))
                })?;
            }
            "file_name" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(format!("Error reading text: {}", e)))?;

                upload_file_request.file_name = text;
            }
            "file" => {
                let file_name = field.file_name().map(str::to_string);
                let content_type = field.content_type().map(|ct| ct.to_string());

                // Read the file bytes (consumes field here)
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Internal(format!("Error reading file bytes: {}", e)))?;

                // Example: Save or process file data
                // fields.file = Some((file_name, content_type, data));
            }
            _ => {}
        }
    }

    println!("{:?}", upload_file_request);

    // let value = serde_json::to_value(&fields).map_err(|e| AppError::Internal(e.to_string()))?;

    if let Err(errors) = upload_file_request.validate() {
        return Err(AppError::Validation(errors));
    }

    Ok("file secret data".to_string())
}
