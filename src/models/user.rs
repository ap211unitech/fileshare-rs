use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{dtos::user::RegisterUserRequest, error::AppError, utils::hash_password};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCollection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub name: String,
    pub email: String,
    pub hashed_password: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<RegisterUserRequest> for UserCollection {
    type Error = AppError;

    fn try_from(payload: RegisterUserRequest) -> Result<Self, Self::Error> {
        let hashed_password = hash_password(&payload.password)
            .map_err(|e| AppError::Internal(format!("Error in hashing password: {}", e)))?;

        let user = UserCollection {
            id: None,
            email: payload.email,
            name: payload.name,
            is_verified: false,
            created_at: Utc::now(),
            hashed_password,
        };

        Ok(user)
    }
}
