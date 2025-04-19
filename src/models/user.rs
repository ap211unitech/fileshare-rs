use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserCollection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub name: String,
    pub email: String,
    pub hashed_password: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}
