use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::Utc;
use validator::Validate;

use crate::{
    config::AppState, dtos::user::RegisterUserRequest, models::user::UserCollection,
    utils::hash_password,
};

pub async fn register_user(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, ()> {
    if let Err(error) = payload.validate() {
        tracing::error!("{}", error.to_string());
    }

    let hashed_password = hash_password(&payload.password);

    let user = UserCollection {
        id: None,
        email: payload.email,
        name: payload.name,
        is_verified: false,
        created_at: Utc::now(),
        hashed_password,
    };

    app_state
        .user_collection
        .insert_one(user)
        .await
        .expect("msg");

    Ok((StatusCode::CREATED, "Sign Up".to_string()))
}
