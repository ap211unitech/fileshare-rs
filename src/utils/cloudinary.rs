use std::collections::BTreeMap;

use chrono::Utc;
use reqwest::{multipart, Client};
use sha1::{Digest, Sha1};

use crate::{config::AppConfig, error::AppError};

/// Saves an encrypted file to the cloudinary
///
/// # Arguments
/// * `encrypted_file` - A reference to the encrypted file bytes.
/// * `file_name` - A base name to include in the output file name.
///
/// # Returns
/// * `Ok(String)` containing the file path of the saved file.
/// * `Err(AppError)` if the directory or file operation fails.
pub async fn upload_file_to_cloud(
    encrypted_file: &[u8],
    file_name: &str,
) -> Result<String, AppError> {
    let app_config = AppConfig::load_config();

    let timestamp = Utc::now().timestamp().to_string();

    // Params to sign
    let mut params = BTreeMap::new();
    params.insert("timestamp", timestamp.clone());

    // Generate signature
    let mut to_sign = String::new();
    for (k, v) in &params {
        to_sign.push_str(&format!("{}={}&", k, v));
    }
    to_sign.pop(); // remove trailing '&'
    to_sign.push_str(&app_config.cloudinary_api_secret);

    let signature = Sha1::digest(to_sign.as_bytes());
    let signature_hex = hex::encode(signature);

    // Create multipart form
    let part = multipart::Part::bytes(encrypted_file.to_vec())
        .file_name(file_name.to_string())
        .mime_str("application/octet-stream")
        .map_err(|e| AppError::Internal(format!("Error creating file stream: {e}")))?;

    let form = multipart::Form::new()
        .part("file", part)
        .text("api_key", app_config.cloudinary_api_key)
        .text("timestamp", timestamp)
        .text("signature", signature_hex);

    // POST to Cloudinary
    let upload_url = format!(
        "https://api.cloudinary.com/v1_1/{}/auto/upload",
        app_config.cloudinary_cloud_name
    );

    let client = reqwest::Client::new();
    let res = client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Error uploading file stream: {e}")))?;

    if res.status().is_success() {
        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Error in parsing json response: {e}")))?;
        let secure_url = json["secure_url"].as_str().unwrap_or_default().to_string();
        tracing::info!("File has been uploaded to cloud");
        Ok(secure_url)
    } else {
        let text = res
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("Error in parsing response: {e}")))?;
        tracing::error!("Error in uploading file to cloud: {}", text);
        Err(AppError::Internal(text))
    }
}

/// Reads raw file data from a public cloud URL (e.g., Cloudinary).
///
/// # Arguments
/// * `url` - A `String` representing the URL of the file to download.
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the file's byte contents if the request is successful.
/// * `Err(AppError)` if the HTTP request fails or the response cannot be converted to bytes.
pub async fn read_file_from_cloud(url: String) -> Result<Vec<u8>, AppError> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Error in fetching file: {e}")))?;

    if response.status().is_success() {
        let bytes = response.bytes().await.map_err(|e| {
            AppError::Internal(format!("Error in converting file data to bytes: {e}"))
        })?;
        Ok(bytes.to_vec())
    } else {
        Err(AppError::Internal(format!(
            "Failed to fetch file: {}",
            response.status()
        )))
    }
}

/// Extracts the public ID from a Cloudinary file URL.
///
/// The public ID is assumed to be the last segment of the URL (after the final `/`).
///
/// # Arguments
/// * `file_url` - A string slice representing the full Cloudinary file URL.
///
/// # Returns
/// * `String` containing the extracted public ID.
fn extract_public_id(file_url: &str) -> String {
    let mut response = String::new();

    for char in file_url.chars().rev() {
        if char == '/' {
            break;
        }
        response.push(char);
    }

    response.chars().rev().collect()
}

/// Deletes a file from Cloudinary using its public URL.
///
/// # Arguments
/// * `file_url` - A `String` representing the Cloudinary file URL to delete.
///
/// # Returns
/// * `Ok(true)` if the deletion is successful.
/// * `Err(AppError)` if the request fails or Cloudinary returns an error.
pub async fn delete_file_from_cloud(file_url: String) -> Result<bool, AppError> {
    let app_config = AppConfig::load_config();

    let timestamp = Utc::now().timestamp().to_string();

    let public_id = extract_public_id(&file_url);

    // Prepare URL for deletion - will automatically handle any file type
    let url = format!(
        "https://api.cloudinary.com/v1_1/{}/raw/destroy",
        app_config.cloudinary_cloud_name
    );

    // Create HTTP client
    let client = Client::new();

    // Params to sign
    let mut params = BTreeMap::new();
    params.insert("timestamp", timestamp.clone());
    params.insert("public_id", public_id.clone());

    // Generate signature
    let mut to_sign = String::new();
    for (k, v) in &params {
        to_sign.push_str(&format!("{}={}&", k, v));
    }
    to_sign.pop(); // remove trailing '&'
    to_sign.push_str(&app_config.cloudinary_api_secret);

    let signature = Sha1::digest(to_sign.as_bytes());
    let signature_hex = hex::encode(signature);

    let form = multipart::Form::new()
        .text("public_id", public_id.clone())
        .text("api_key", app_config.cloudinary_api_key)
        .text("timestamp", timestamp)
        .text("signature", signature_hex);

    // Perform DELETE request
    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Error deleting file: {e}")))?;

    if response.status().is_success() {
        tracing::info!("Successfully deleted: {}", public_id);
        Ok(true)
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("Error in parsing response: {e}")))?;
        tracing::error!("Failed to delete file: {} — {}", status, body);
        Err(AppError::Internal(format!(
            "Failed to delete file: {} — {}",
            status, body
        )))
    }
}
