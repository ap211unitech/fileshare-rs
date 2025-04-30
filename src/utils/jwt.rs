use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use super::extractor::ExtractAuthAgent;
use crate::{config::AppConfig, error::AppError};

/// Struct representing the JWT claims payload.
/// Includes:
/// - `user_id`: Unique identifier for the user (MongoDB ObjectId).
/// - `exp`: Expiration timestamp (as a Unix timestamp).
/// - `iat`: Issued-at timestamp (as a Unix timestamp).
#[derive(Serialize)]
pub struct JwtClaim {
    pub user_id: ObjectId,
    pub exp: usize,
    pub iat: usize,
}

/// Encodes a JWT using the user's ID and application secret key.
///
/// # Arguments
/// * `user_id` - MongoDB ObjectId representing the authenticated user.
///
/// # Returns
/// * `Ok(String)` with the encoded JWT if successful.
/// * `Err(AppError)` if encoding fails.
pub fn encode_jwt(user_id: ObjectId) -> Result<String, AppError> {
    let app_config = AppConfig::load_config();

    let iat = Utc::now();
    let expire = Duration::hours(24); // expire after 1 day

    let jwt_claim = JwtClaim {
        user_id,
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

/// Decodes and validates a JWT token using the application's secret key.
///
/// # Arguments
/// * `jwt_token` - The encoded JWT string to be decoded and validated.
///
/// # Returns
/// * `Ok(TokenData<ExtractAuthAgent>)` if the token is valid and not expired.
/// * `Err(AppError)` if the token is invalid or expired.
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
