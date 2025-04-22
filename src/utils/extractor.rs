use axum::extract::FromRequestParts;
use reqwest::header;
use serde::Deserialize;

use super::jwt::decode_jwt;
use crate::error::AppError;

#[derive(Deserialize)]
pub struct ExtractAuthAgent {
    pub email: String,
}

impl<S> FromRequestParts<S> for ExtractAuthAgent
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Get AUTHORIZATION header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        // Try converting it to string
        let auth_str = auth_header
            .to_str()
            .map_err(|_| AppError::BadRequest("Invalid Authorization header format".to_string()))?;

        // Parse `token` field
        let jwt_token = auth_str
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Expected Bearer token".to_string()))?;

        let token_data = decode_jwt(jwt_token)?;

        Ok(ExtractAuthAgent {
            email: token_data.claims.email,
        })
    }
}
