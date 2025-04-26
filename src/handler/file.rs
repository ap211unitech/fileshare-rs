use axum::{extract::Multipart, response::IntoResponse, Extension};
use chrono::DateTime;
use validator::Validate;

use crate::{
    config::AppState,
    dtos::file::UploadFileRequest,
    error::AppError,
    models::file::FileCollection,
    utils::{
        extractor::ExtractAuthAgent,
        file::{encrypt_file_with_password, upload_file_to_server},
    },
};

pub async fn upload_file(
    agent: ExtractAuthAgent,
    Extension(app_state): Extension<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut upload_file_request = UploadFileRequest::default();
    upload_file_request.user_id = agent.user_id;

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
            "password" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(format!("Error reading text: {}", e)))?;

                upload_file_request.password = Some(text);
            }
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
                let content_type = field
                    .content_type()
                    .map(|ct| ct.to_string())
                    .ok_or_else(|| AppError::Internal(format!("Error reading file type")))?;

                // Read the file bytes (consumes field here)
                let file_data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Internal(format!("Error reading file bytes: {}", e)))?;

                upload_file_request.mime_type = content_type;
                upload_file_request.file_data = file_data;
            }
            _ => {}
        }
    }

    if let Err(errors) = upload_file_request.validate() {
        return Err(AppError::Validation(errors));
    }

    let encrypted_file = encrypt_file_with_password(
        upload_file_request.file_data.to_vec(),
        &upload_file_request
            .password
            .clone()
            .unwrap_or("default-password".to_string()),
    )?;

    upload_file_request.cid =
        upload_file_to_server(&encrypted_file, &upload_file_request.file_name)?;

    let file = FileCollection::from(upload_file_request.clone());

    Ok("file secret data".to_string())
}
