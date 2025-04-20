use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use validator::Validate;

use crate::{
    config::AppState,
    dtos::user::{RegisterUserRequest, RegisterUserResponse},
    error::AppError,
    models::user::UserCollection,
    utils::get_inserted_id,
};

pub async fn register_user(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::Validation(errors));
    }

    let user = UserCollection::from(payload);

    let user_doc = app_state
        .user_collection
        .insert_one(user)
        .await
        .map_err(|e| AppError::Database(e))?;

    let id = get_inserted_id(&user_doc)?;

    tracing::info!("{:?}", user_doc);

    Ok((StatusCode::CREATED, Json(RegisterUserResponse { id })))
}
