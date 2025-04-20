use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use mongodb::bson::doc;
use validator::Validate;

use crate::{
    config::AppState,
    dtos::user::{LoginUserRequest, LoginUserResponse, RegisterUserRequest, RegisterUserResponse},
    error::AppError,
    models::user::UserCollection,
    utils::{get_inserted_id, send_email, verify_password, SendgridUser},
};

pub async fn register_user(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::Validation(errors));
    }

    send_email(SendgridUser {
        name: &payload.name,
        email: &payload.email,
    })
    .await?;

    let user = UserCollection::try_from(payload)?;

    let user_doc = app_state
        .user_collection
        .insert_one(user)
        .await
        .map_err(|e| AppError::Database(e))?;

    let id = get_inserted_id(&user_doc)?;

    tracing::info!("{:?}", user_doc);

    Ok((StatusCode::CREATED, Json(RegisterUserResponse { id })))
}

pub async fn login_user(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<LoginUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = app_state
        .user_collection
        .find_one(doc! {"email": payload.email})
        .await?
        .ok_or_else(|| AppError::BadRequest("No such user exists!".to_string()))?;

    let is_valid_password = verify_password(&user.hashed_password, &payload.password)
        .map_err(|e| AppError::Internal(e))?;

    if !is_valid_password {
        return Err(AppError::BadRequest("Password do not match!".to_string()));
    }

    println!("{:?}", user);

    Ok((
        StatusCode::OK,
        Json(LoginUserResponse {
            token: "token".to_string(),
        }),
    ))
}
