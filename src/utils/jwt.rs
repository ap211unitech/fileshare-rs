use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::Serialize;

use super::extractor::ExtractAuthAgent;
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

pub fn decode_jwt(jwt_token: &str) -> Result<TokenData<ExtractAuthAgent>, AppError> {
    let app_config = AppConfig::load_config();

    // Enable expiration check to reject tokens that are no longer valid
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true; // ✅ Ensure expiration is checked

    let token_data = decode::<ExtractAuthAgent>(
        jwt_token,
        &DecodingKey::from_secret(app_config.jwt_secret_key.as_ref()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(token_data)
}
