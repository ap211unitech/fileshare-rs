use std::collections::HashMap;

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use validator::Validate;

use crate::{
    config::{AppConfig, AppState},
    dtos::user::{
        LoginUserRequest, LoginUserResponse, RegisterUserRequest, RegisterUserResponse,
        VerifyUserResponse,
    },
    error::AppError,
    models::{
        token::{TokenCollection, TokenInfo, TokenType},
        user::UserCollection,
    },
    utils::{email::EmailInfo, hashing::verify_secret, jwt::encode_jwt, misc::get_inserted_id},
};

pub async fn register_user(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::Validation(errors));
    }

    let app_config = AppConfig::load_config();

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
        user_id: user_doc.inserted_id.clone(),
    };

    let token = TokenCollection::try_from(email_verification_info.clone())?;

    let token_doc = app_state
        .token_collection
        .insert_one(token)
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!("Token Doc: {:?}", token_doc);

    let user_object_id = get_inserted_id(&user_doc)?;

    // Send email to user
    EmailInfo {
        recipient_name: &payload.name,
        recipient_email: &payload.email,
        email_type: TokenType::EmailVerification,
        verification_link: &format!(
            "{SERVER_URL}/user/verify?token={VERIFICATION_TOKEN}&user={USER_ID}",
            SERVER_URL = app_config.server_url,
            VERIFICATION_TOKEN = email_verification_info.token,
            USER_ID = user_object_id
        ),
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

pub async fn verify_user(
    Query(info): Query<HashMap<String, String>>,
    Extension(app_state): Extension<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let (verification_token, user_id) = (
        info.get("token")
            .ok_or_else(|| AppError::BadRequest("`token` query not given".to_string()))?,
        info.get("user")
            .ok_or_else(|| AppError::BadRequest("`user` query not given".to_string()))?,
    );

    // Convert user_id into ObjectId
    let user_id = ObjectId::parse_str(user_id)
        .map_err(|_| AppError::BadRequest("Invalid `user` id format".to_string()))?;

    // Find appropriate token
    let token = app_state
        .token_collection
        .find_one(doc! {
            "token_type": TokenType::EmailVerification.to_string(),
            "user_id": user_id
        })
        .await?
        .ok_or_else(|| AppError::BadRequest("No token exists for given user!".to_string()))?;

    // Check if valid user exists
    let user = app_state
        .user_collection
        .find_one(doc! {"_id": token.user_id})
        .await?
        .ok_or_else(|| AppError::BadRequest("No such user exists!".to_string()))?;

    tracing::info!("User verification attempt: user={:?}", user,);

    // Check if token is not expired
    if token.expires_at < Utc::now() {
        return Err(AppError::BadRequest("Token expired!".to_string()));
    }

    // Check if user is not already verified
    if user.is_verified {
        return Err(AppError::BadRequest("User already verified!".to_string()));
    }

    // Check if token is correct
    let is_valid_token = verify_secret(&token.hashed_token, &verification_token)?;
    if !is_valid_token {
        return Err(AppError::BadRequest("Invalid token provided!".to_string()));
    }

    // Mark user resolved and delete the token info
    app_state
        .user_collection
        .update_one(doc! {"_id": user_id}, doc! {"$set": {"is_verified": true}})
        .await?;

    app_state
        .token_collection
        .delete_one(doc! {"_id": token.id})
        .await?;

    Ok((
        StatusCode::OK,
        Json(VerifyUserResponse {
            message: "User verification successful".to_string(),
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

    let is_valid_password = verify_secret(&user.hashed_password, &payload.password)?;

    // Check if password is valid
    if !is_valid_password {
        return Err(AppError::BadRequest("Password do not match!".to_string()));
    }

    // Check if user is verified
    if !user.is_verified {
        return Err(AppError::BadRequest(
            "Your account is not verified yet!".to_string(),
        ));
    }

    // Get user_id safely
    let user_id = user
        .id
        .ok_or(AppError::BadRequest("Invalid user!".to_string()))?;

    // Generate JWT token
    let token = encode_jwt(user_id)?;

    tracing::info!("User logging in: {:?}", user);

    Ok((StatusCode::OK, Json(LoginUserResponse { token })))
}
