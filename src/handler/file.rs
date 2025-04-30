use std::fs;

use axum::{
    body::Body,
    extract::{Multipart, Query},
    http::{HeaderMap, HeaderValue, Response},
    response::IntoResponse,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::bson::doc;
use reqwest::StatusCode;
use validator::Validate;

use crate::{
    config::AppState,
    dtos::file::{DownloadFileRequest, UploadFileRequest, UploadFileResponse, UserFilesResponse},
    error::AppError,
    models::file::FileCollection,
    utils::{
        extractor::ExtractAuthAgent,
        file::{decrypt_file_with_password, encrypt_file_with_password, upload_file_to_server},
        misc::{object_id_to_str, str_to_object_id},
    },
};

/// Handles authenticated file uploads via multipart/form-data.
///
/// Accepts the following fields:
/// - `file` (required): The file to be uploaded.
/// - `file_name`: A user-defined name for the file.
/// - `password` (required): Used to encrypt the file before storage.
/// - `expires_at`: ISO datetime for file expiration.
/// - `max_downloads` (optional): Max number of allowed downloads.
///
/// The file is encrypted using the provided password, uploaded to storage,
/// and metadata is saved to MongoDB. Returns a file ID on success.
///
/// # Parameters
/// - `agent`: Authenticated user context.
/// - `app_state`: Shared application state with DB references.
/// - `multipart`: Incoming multipart form data.
///
/// # Returns
/// - `201 Created` with JSON `{ message, id }` on success.
/// - `AppError` variants for validation, parsing, encryption, or DB errors.
///
/// # Security
/// File contents are encrypted at upload; passwords are not stored.
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

                upload_file_request.password = text;
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

                upload_file_request.max_downloads = text.parse::<u8>().map_err(|e| {
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

                upload_file_request.size = file_data.len() as u64;
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
        &upload_file_request.password,
    )?;

    upload_file_request.cid =
        upload_file_to_server(&encrypted_file, &upload_file_request.file_name)?;

    tracing::info!("File uploaded to server");

    let file = FileCollection::from(upload_file_request.clone());

    let uploaded_file_result = app_state.file_collection.insert_one(file).await?;

    tracing::info!("File metadata uploaded to database");

    Ok((
        StatusCode::CREATED,
        Json(UploadFileResponse {
            message: "File uploaded successfully".to_string(),
            id: object_id_to_str(&uploaded_file_result.inserted_id.as_object_id())?,
        }),
    ))
}

/// Handles secure file downloads based on file ID and optional password.
///
/// Accepts a JSON payload with `file_id` and an optional `password`.
/// Verifies the file's existence, expiration, and download limits before
/// decrypting and returning the file as a downloadable attachment.
///
/// # Parameters
/// - `app_state`: Shared application state with DB and file access.
/// - `payload`: JSON body containing the file ID and optional password.
///
/// # Returns
/// - `200 OK` with the decrypted file and appropriate headers on success.
/// - `AppError` on invalid ID, missing file, decryption failure, or limits exceeded.
///
/// # Security
/// Files are encrypted at rest and decrypted using the provided or default password.
///
/// # Example
/// ```http
/// GET /file/download?file_id=6811a257200ffe8eb047b776&password=12345
/// ```
pub async fn download_file(
    Extension(app_state): Extension<AppState>,
    Query(query): Query<DownloadFileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let file_id = str_to_object_id(&query.file_id)?;
    let password = match query.password {
        Some(password) => password,
        None => String::from("default-password"),
    };

    // get file
    let file = app_state
        .file_collection
        .find_one(doc! {"_id": file_id})
        .await?
        .ok_or_else(|| AppError::BadRequest("No such file exists!".to_string()))?;

    // check expiry date and download count
    if file.expires_at <= Utc::now() {
        return Err(AppError::BadRequest(
            "File has already expired.".to_string(),
        ));
    }

    if file.download_count >= file.max_downloads {
        return Err(AppError::BadRequest(
            "File has reached its maximum download limit.".to_string(),
        ));
    }

    // decrypt file
    let encrypted_file = fs::read(file.cid)
        .map_err(|e| AppError::BadRequest(format!("Error reading file content: {}", e)))?;

    let decrypted_file = decrypt_file_with_password(&encrypted_file, &password)?;

    // increase download count
    app_state
        .file_collection
        .update_one(
            doc! {"_id": file_id},
            doc! {"$set": {"download_count": (file.download_count + 1) as i32 }},
        )
        .await?;

    let mime_type = file.mime_type;

    // Set headers
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_str(&mime_type).unwrap());
    headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file.name)).unwrap(),
    );

    let mut response_builder = Response::builder().status(StatusCode::OK);
    if let Some(headers_mut) = response_builder.headers_mut() {
        headers_mut.extend(headers);
    }

    let response = response_builder
        .body(Body::from(decrypted_file))
        .map_err(|e| AppError::Internal(format!("Error in download file : {}", e)))?;

    Ok(response)
}

pub async fn user_files(
    agent: ExtractAuthAgent,
    Extension(app_state): Extension<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let mut files = app_state
        .file_collection
        .find(doc! {"user": agent.user_id})
        .await?;

    let mut response = Vec::<FileCollection>::new();

    while let Some(file) = files
        .try_next()
        .await
        .map_err(|e| AppError::Internal(format!("Error in fetching user files{}", e)))?
    {
        response.push(file);
    }

    Ok((StatusCode::OK, Json(UserFilesResponse { files: response })))
}
