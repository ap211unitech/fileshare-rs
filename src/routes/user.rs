use axum::{
    routing::{get, post, put},
    Router,
};

use crate::handler::user::{
    forgot_password, login_user, register_user, reset_password, send_user_verification_email,
    verify_user,
};

pub fn get_user_routes() -> Router {
    Router::new()
        .route("/register", post(register_user))
        .route(
            "/send-verification-email",
            post(send_user_verification_email),
        )
        .route("/login", post(login_user))
        .route("/verify", get(verify_user))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", put(reset_password))
}
