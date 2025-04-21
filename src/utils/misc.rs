use mongodb::results::InsertOneResult;

use crate::error::AppError;

pub fn get_inserted_id(doc: &InsertOneResult) -> Result<String, AppError> {
    Ok(doc
        .inserted_id
        .as_object_id()
        .ok_or_else(|| AppError::Internal("Cannot get inserted id".to_string()))?
        .to_hex())
}
