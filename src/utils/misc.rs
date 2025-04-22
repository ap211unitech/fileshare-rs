use mongodb::bson::oid::ObjectId;

use crate::error::AppError;

pub fn object_id_to_str(object_id: &Option<ObjectId>) -> Result<String, AppError> {
    Ok(object_id
        .ok_or_else(|| AppError::Internal("Cannot get inserted id".to_string()))?
        .to_hex())
}
