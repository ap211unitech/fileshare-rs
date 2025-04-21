use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use mongodb::bson::doc;
use validator::Validate;

use crate::{
    config::AppState,
    dtos::user::{LoginUserRequest, LoginUserResponse, RegisterUserRequest, RegisterUserResponse},
    error::AppError,
    models::{
        token::{TokenCollection, TokenInfo, TokenType},
        user::UserCollection,
    },
    utils::{email::EmailInfo, hashing::verify_password},
};

pub async fn register_user(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::Validation(errors));
    }

    // Create User Info
    let user = UserCollection::try_from(payload.clone())?;

    let user_doc = app_state
        .user_collection
        .insert_one(user)
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!("User Doc: {:?}", user_doc);

    // Generate email verification info
    let email_verification_info = TokenInfo {
        token: uuid::Uuid::new_v4().to_string(),
        token_type: TokenType::EmailVerification,
        user_id: user_doc.inserted_id,
    };

    let token = TokenCollection::try_from(email_verification_info)?;

    let token_doc = app_state
        .token_collection
        .insert_one(token)
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!("Token Doc: {:?}", token_doc);

    // Send email to user
    EmailInfo {
        recipient_name: &payload.name,
        recipient_email: &payload.email,
        email_type: TokenType::EmailVerification,
    }
    .send_email()
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterUserResponse {
            message: "Please check your email. A verification link has been sent to you."
                .to_string(),
        }),
    ))
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
