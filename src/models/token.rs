use chrono::{DateTime, Duration, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::{error::AppError, utils::hashing::hash_secret};

#[derive(Clone)]
pub struct TokenInfo {
    pub user_id: Option<ObjectId>,
    pub token: String,
    pub token_type: TokenType,
}

#[derive(Serialize, Deserialize, Display, Clone)]
pub enum TokenType {
    EmailVerification,
}

#[derive(Serialize, Deserialize)]
pub struct TokenCollection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub token_type: TokenType,
    pub hashed_token: String,

    pub user_id: ObjectId,

    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl TryFrom<TokenInfo> for TokenCollection {
    type Error = AppError;

    fn try_from(payload: TokenInfo) -> Result<Self, Self::Error> {
        let hashed_token = hash_secret(&payload.token)?;

        Ok(TokenCollection {
            id: None,
            hashed_token,
            token_type: payload.token_type,
            user_id: payload
                .user_id
                .ok_or_else(|| AppError::Internal("Cannot parse ObjectId".to_string()))?,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(30), // 30 mins expiration time
        })
    }
}
