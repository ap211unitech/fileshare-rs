use axum::{http::StatusCode, response::IntoResponse, Json};
use validator::Validate;

use crate::dtos::user::RegisterUserRequest;

pub async fn register_user(
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, ()> {
    if let Err(error) = payload.validate() {
        tracing::error!("{}", error.to_string());
    }

    Ok((StatusCode::CREATED, "Sign Up".to_string()))
}
