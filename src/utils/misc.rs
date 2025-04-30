use mongodb::bson::oid::ObjectId;

use crate::error::AppError;

/// Converts an `Option<ObjectId>` to its hexadecimal string representation.
///
/// # Arguments
/// * `object_id` - A reference to an `Option<ObjectId>`. This must contain a valid ObjectId.
///
/// # Returns
/// * `Ok(String)` if the ObjectId is present and successfully converted.
/// * `Err(AppError::Internal)` if the ObjectId is `None`.
pub fn object_id_to_str(object_id: &Option<ObjectId>) -> Result<String, AppError> {
    Ok(object_id
        .ok_or_else(|| AppError::Internal("Can not parse object_id to string".to_string()))?
        .to_hex())
}

/// Parses a string into a MongoDB `ObjectId`.
///
/// # Arguments
/// * `str` - A string slice expected to be a valid 24-character hex representation of an ObjectId.
///
/// # Returns
/// * `Ok(ObjectId)` if the input string is valid.
/// * `Err(AppError::BadRequest)` if the input string is not a valid ObjectId.
pub fn str_to_object_id(str: &str) -> Result<ObjectId, AppError> {
    let object_id = ObjectId::parse_str(str)
        .map_err(|_| AppError::BadRequest("Can not parse string to object_id".to_string()))?;

    Ok(object_id)
}
