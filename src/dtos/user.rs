use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,

    #[validate(email(message = "Invalid email"))]
    pub email: String,

    #[validate(length(min = 5, message = "Password should be atleast 5 characters long"))]
    pub password: String,

    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    pub confirm_password: String,
}

#[derive(Serialize)]
pub struct RegisterUserResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct VerifyUserResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginUserResponse {
    pub token: String,
}

#[derive(Deserialize, Validate)]
pub struct SendUserVerificationEmailRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
}

#[derive(Serialize)]
pub struct SendUserVerificationEmailResponse {
    pub message: String,
}

#[derive(Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
}

#[derive(Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 5, message = "Password should be atleast 5 characters long"))]
    pub new_password: String,

    #[validate(must_match(other = "new_password", message = "Passwords do not match"))]
    pub confirm_new_password: String,
}

#[derive(Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}
