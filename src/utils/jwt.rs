use axum::extract::FromRequestParts;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::{config::AppConfig, error::AppError};

#[derive(Serialize)]
pub struct JwtClaim {
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn encode_jwt(email: &str) -> Result<String, AppError> {
    let app_config = AppConfig::load_config();

    let iat = Utc::now();
    let expire = Duration::hours(24); // expire after 1 day

    let jwt_claim = JwtClaim {
        email: email.to_string(),
        iat: iat.timestamp() as usize,
        exp: (iat + expire).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &jwt_claim,
        &EncodingKey::from_secret(app_config.jwt_secret_key.as_ref()),
    )
    .map_err(|e| AppError::Jwt(e.to_string()))?;

    Ok(token)
}

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
        let app_config = AppConfig::load_config();

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

        // Enable expiration check to reject tokens that are no longer valid
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true; // ✅ Ensure expiration is checked

        let token_data = decode::<ExtractAuthAgent>(
            jwt_token,
            &DecodingKey::from_secret(app_config.jwt_secret_key.as_ref()),
            &validation,
        )
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

        Ok(ExtractAuthAgent {
            email: token_data.claims.email,
        })
    }
}
