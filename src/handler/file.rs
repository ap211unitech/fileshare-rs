use axum::{extract::Multipart, response::IntoResponse};

use crate::utils::extractor::ExtractAuthAgent;

pub async fn upload_file(agent: ExtractAuthAgent, mut multipart: Multipart) -> impl IntoResponse {
    println!("{:?}", agent);
    "file secret data".to_string()
}
