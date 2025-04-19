use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    name: String,

    #[validate(email(message = "Invalid email"))]
    email: String,

    #[validate(length(min = 5, message = "Password should be atleast 5 characters long"))]
    password: String,

    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    confirm_password: String,
}
