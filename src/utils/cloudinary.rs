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
