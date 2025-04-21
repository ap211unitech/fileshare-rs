use axum::{
    routing::{get, post},
    Router,
};

use crate::handler::user::{login_user, register_user, verify_user};

pub fn get_user_routes() -> Router {
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/verify", get(verify_user))
}
