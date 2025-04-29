use std::collections::HashMap;

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{Duration, Utc};
use mongodb::bson::doc;
use validator::Validate;

use crate::{
    config::{AppConfig, AppState},
    dtos::user::{
        ForgotPasswordRequest, ForgotPasswordResponse, LoginUserRequest, LoginUserResponse,
        RegisterUserRequest, RegisterUserResponse, ResetPasswordRequest, ResetPasswordResponse,
        SendUserVerificationEmailRequest, SendUserVerificationEmailResponse, VerifyUserResponse,
    },
    error::AppError,
    models::{
        token::{TokenCollection, TokenInfo, TokenType},
        user::UserCollection,
    },
    utils::{
        email::EmailInfo,
        hashing::{hash_secret, verify_secret},
        jwt::encode_jwt,
        misc::{object_id_to_str, str_to_object_id},
    },
};

const TOKEN_COOLDOWN_MINUTES: i64 = 5;

/// Registers a new user with the provided email and credentials.
///
/// Accepts a JSON payload with user registration details. Validates the input,
/// creates a new user record, and stores it in the database. Returns a success
/// message upon successful registration.
///
/// # Parameters
/// - `app_state`: Shared application state containing the user collection.
/// - `payload`: JSON body with user registration data (e.g., email, password).
///
/// # Returns
/// - `201 Created` with a confirmation message on success.
/// - `AppError::Validation` for invalid input.
/// - `AppError::Database` if user insertion fails.
///
/// # Example
/// ```http
/// POST /user/register/
/// ```
/// ```json
/// {
///   "name": "user",
///   "email": "user@example.com",
///   "password": "securePassword123",
///   "confirm_password": "securePassword123"
/// }
/// `
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

    Ok((
        StatusCode::CREATED,
        Json(RegisterUserResponse {
            message: "Email registered successfully! Please verify your email now.".to_string(),
        }),
    ))
}

/// Sends a verification email to a registered, unverified user.
///
/// Accepts a JSON payload containing the user's email. If the user exists and is not
/// yet verified, a verification token is generated and emailed. A cooldown of 5 minutes
/// is enforced between successive token requests.
///
/// # Parameters
/// - `app_state`: Shared application state with user and token collections.
/// - `payload`: JSON body with the user's email address.
///
/// # Returns
/// - `200 OK` with a message indicating the email has been sent.
/// - `AppError::Validation` for invalid input.
/// - `AppError::BadRequest` if the user doesn't exist, is already verified, or if a token was recently sent.
/// - `AppError::Database` for DB operation failures.
///
/// # Example
/// ```http
/// POST /user/send-verification-email/
/// ```
/// ```json
/// {
///   "email": "user@example.com"
/// }
/// ```
pub async fn send_user_verification_email(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<SendUserVerificationEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::Validation(errors));
    }

    let app_config = AppConfig::load_config();

    // Check if user exists for given email
    let user = app_state
        .user_collection
        .find_one(doc! {"email": &payload.email})
        .await?
        .ok_or_else(|| AppError::BadRequest("No such user exists!".to_string()))?;

    if user.is_verified {
        return Err(AppError::BadRequest("User already verified!".to_string()));
    }

    // Check if there is already a email verification token for this user
    let token = app_state
        .token_collection
        .find_one(doc! {
            "token_type": TokenType::EmailVerification.to_string(),
            "user_id": user.id
        })
        .await?;

    // If token already exists
    if let Some(token) = token {
        let current_timestamp = Utc::now();
        let next_token_should_be_send_at =
            token.created_at + Duration::minutes(TOKEN_COOLDOWN_MINUTES); // 5-minute cooldown period

        // If the request is made before the cooldown period ends, return an error
        if next_token_should_be_send_at > current_timestamp {
            return Err(AppError::BadRequest(
                "Next request can be made after 5 minutes only".to_string(),
            ));
        }

        // Cooldown period has passed; delete the existing token
        app_state
            .token_collection
            .delete_one(doc! {"_id": token.id})
            .await?;
    }

    // Generate email verification info
    let email_verification_info = TokenInfo {
        token: uuid::Uuid::new_v4().to_string(),
        token_type: TokenType::EmailVerification,
        user_id: user.id.clone(),
    };

    let token = TokenCollection::try_from(email_verification_info.clone())?;

    let token_doc = app_state
        .token_collection
        .insert_one(token)
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!("Token Doc: {:?}", token_doc);

    // Convert user_id into String
    let user_object_id_as_str = object_id_to_str(&user.id)?;

    // Spawn an asynchronous task to send the email in the background
    // This task creates an EmailInfo instance with the necessary information and sends the email asynchronously.
    tokio::spawn(async move {
        EmailInfo {
            recipient_email: &payload.email,
            email_type: TokenType::EmailVerification,
            verification_link: &format!(
                "{SERVER_URL}/user/verify?token={VERIFICATION_TOKEN}&user={USER_ID}",
                SERVER_URL = app_config.server_url,
                VERIFICATION_TOKEN = email_verification_info.token,
                USER_ID = user_object_id_as_str
            ),
        }
        .send_email()
        .await
    });

    Ok((
        StatusCode::OK,
        Json(SendUserVerificationEmailResponse {
            message: "Please check your email. A verification link has been sent to you."
                .to_string(),
        }),
    ))
}

/// Verifies a user's email using a token provided via query parameters.
///
/// Expects `token` and `user` (user ID) as query parameters. Validates the token,
/// checks for expiration, and ensures the user is not already verified. If valid,
/// the user's verification status is updated and the token is deleted.
///
/// # Parameters
/// - `info`: Query parameters containing `token` and `user`.
/// - `app_state`: Shared application state with user and token collections.
///
/// # Returns
/// - `200 OK` with a success message if verification is successful.
/// - `AppError::BadRequest` for missing or invalid query params, expired tokens, or invalid users.
/// - `AppError::Internal` or `Database` for server or DB-related issues.
///
/// # Example
/// ```http
/// GET /user/verify?token=abc123&user=605c72afee3a3a9b2c9d8d91
/// ```
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
    let user_id = str_to_object_id(user_id)?;

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

/// Authenticates a user and generates a JWT token for successful login.
///
/// Accepts a JSON payload with the user's email and password. Verifies the user's credentials,
/// checks if the user is verified, and generates a JWT token if the login is successful.
///
/// # Parameters
/// - `app_state`: Shared application state with user collection.
/// - `payload`: JSON body containing the user's `email` and `password`.
///
/// # Returns
/// - `200 OK` with a JWT token in the response body if login is successful.
/// - `AppError::BadRequest` if the user does not exist, the password is incorrect, or the user is not verified.
///
/// # Example
/// ```http
/// POST /user/login/
/// ```
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "password123"
/// }
/// ```
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
        return Err(AppError::BadRequest("Wrong Password!".to_string()));
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

/// Initiates the password reset process by sending a reset link to the user's email.
///
/// Accepts a JSON payload containing the user's email, validates the email, checks if the user exists,
/// verifies that the email is confirmed, and generates a "forgot password" token. An email is sent to the
/// user with a password reset link if the request is valid.
///
/// # Parameters
/// - `app_state`: Shared application state containing the user and token collections.
/// - `payload`: JSON body containing the user's `email`.
///
/// # Returns
/// - `200 OK` with a success message if the email is valid and the reset link has been sent.
/// - `AppError::BadRequest` if the user does not exist, the email is not verified, or if a token is already active.
/// - `AppError::Validation` if the input data fails validation.
///
/// # Example
/// ```http
/// POST /user/forgot-password/
/// ```
/// ```json
/// {
///   "email": "user@example.com"
/// }
/// ```
pub async fn forgot_password(
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::Validation(errors));
    }

    let app_config = AppConfig::load_config();

    // Find user
    let user = app_state
        .user_collection
        .find_one(doc! {"email": &payload.email})
        .await?
        .ok_or_else(|| AppError::BadRequest("No such user exists!".to_string()))?;

    // Check if email is verified
    if !user.is_verified {
        return Err(AppError::BadRequest(
            "Please verify your email first".to_string(),
        ));
    }

    // Check if there is already a forgot password token for this user
    let token = app_state
        .token_collection
        .find_one(doc! {
            "token_type": TokenType::ForgotPassword.to_string(),
            "user_id": user.id
        })
        .await?;

    // If token already exists
    if let Some(token) = token {
        let current_timestamp = Utc::now();
        let next_token_should_be_send_at =
            token.created_at + Duration::minutes(TOKEN_COOLDOWN_MINUTES); // 5-minute cooldown period

        // If the request is made before the cooldown period ends, return an error
        if next_token_should_be_send_at > current_timestamp {
            return Err(AppError::BadRequest(
                "Next request can be made after 5 minutes only".to_string(),
            ));
        }

        // Cooldown period has passed; delete the existing token
        app_state
            .token_collection
            .delete_one(doc! {"_id": token.id})
            .await?;
    }

    // Generate forgot password verification info
    let forgot_password_info = TokenInfo {
        token: uuid::Uuid::new_v4().to_string(),
        token_type: TokenType::ForgotPassword,
        user_id: user.id,
    };

    // Create TokenCollection document
    let token = TokenCollection::try_from(forgot_password_info.clone())?;

    // Insert token info in database
    let token_doc = app_state
        .token_collection
        .insert_one(token)
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!("Forgot password token doc: {:?}", token_doc);

    let user_object_id_as_str = object_id_to_str(&user.id)?;

    // Spawn an asynchronous task to send the email in the background
    // This task creates an EmailInfo instance with the necessary information and sends the email asynchronously.
    tokio::spawn(async move {
        EmailInfo {
            recipient_email: &payload.email,
            email_type: TokenType::ForgotPassword,
            verification_link: &format!(
                "{SERVER_URL}/user/reset-password?token={VERIFICATION_TOKEN}&user={USER_ID}",
                SERVER_URL = app_config.server_url,
                VERIFICATION_TOKEN = forgot_password_info.token,
                USER_ID = user_object_id_as_str
            ),
        }
        .send_email()
        .await
    });

    Ok((
        StatusCode::OK,
        Json(ForgotPasswordResponse {
            message: "Please check your email. A link has been sent to you for reset password."
                .to_string(),
        }),
    ))
}

/// Resets the password for the user by validating the provided token and setting the new password.
///
/// This function checks if the `forgot_password_token` and `user_id` parameters are valid, verifies the token
/// validity, and updates the user's password in the database if everything checks out. The token is then deleted
/// after a successful password reset.
///
/// # Parameters
/// - `query`: Contains the query parameters `token` (forgot password token) and `user` (user ID).
/// - `app_state`: Shared application state containing the user and token collections.
/// - `payload`: JSON body containing the user's new password (`new_password`).
///
/// # Returns
/// - `200 OK` with a success message if the password was successfully reset.
/// - `AppError::BadRequest` if the token is invalid, expired, or the user does not exist.
/// - `AppError::Validation` if the input data fails validation.
///
/// # Example
/// ```http
/// POST /user/reset-password?token=abc123&user=605c72afee3a3a9b2c9d8d91
/// ```
/// ```json
/// {
///   "new_password": "newSecurePassword123"
///   "confirm_new_password": "newSecurePassword123"
/// }
/// ```
pub async fn reset_password(
    Query(query): Query<HashMap<String, String>>,
    Extension(app_state): Extension<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let (forgot_password_token, user_id) = (
        query
            .get("token")
            .ok_or_else(|| AppError::BadRequest("`token` query not given".to_string()))?,
        query
            .get("user")
            .ok_or_else(|| AppError::BadRequest("`user` query not given".to_string()))?,
    );

    if let Err(errors) = payload.validate() {
        return Err(AppError::Validation(errors));
    }

    // Convert user_id into ObjectId
    let user_id = str_to_object_id(&user_id.to_string())?;

    // Find appropriate token
    let token = app_state
        .token_collection
        .find_one(doc! {
            "token_type": TokenType::ForgotPassword.to_string(),
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

    tracing::info!("Reset password attempt: user={:?}", user);

    // Check if token is not expired
    if token.expires_at < Utc::now() {
        return Err(AppError::BadRequest("Token expired!".to_string()));
    }

    // Check if token is correct
    let is_valid_token = verify_secret(&token.hashed_token, &forgot_password_token)?;
    if !is_valid_token {
        return Err(AppError::BadRequest("Invalid token provided!".to_string()));
    }

    // Set new password and delete the token info
    let new_hashed_password = hash_secret(&payload.new_password)?;
    app_state
        .user_collection
        .update_one(
            doc! {"_id": user_id},
            doc! {"$set": {"hashed_password": new_hashed_password}},
        )
        .await?;

    app_state
        .token_collection
        .delete_one(doc! {"_id": token.id})
        .await?;

    tracing::info!("Password reset successful user={:?}", user);

    Ok((
        StatusCode::OK,
        Json(ResetPasswordResponse {
            message: "Password has been reset successfully".to_string(),
        }),
    ))
}
