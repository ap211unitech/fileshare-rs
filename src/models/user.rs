use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserCollection {
    pub _id: ObjectId,

    pub name: String,
    pub email: String,
    pub hashed_password: String,

    pub created_at: DateTime<Utc>,
}
