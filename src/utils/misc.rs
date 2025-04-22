use mongodb::bson::oid::ObjectId;

use crate::error::AppError;

pub fn object_id_to_str(object_id: &Option<ObjectId>) -> Result<String, AppError> {
    Ok(object_id
        .ok_or_else(|| AppError::Internal("Can not parse object_id to string".to_string()))?
        .to_hex())
}

pub fn str_to_object_id(str: &str) -> Result<ObjectId, AppError> {
    let object_id = ObjectId::parse_str(str)
        .map_err(|_| AppError::BadRequest("Can not parse string to object_id".to_string()))?;

    Ok(object_id)
}
