use axum::{routing::post, Router};

use crate::handler::user::register_user;

pub fn get_user_routes() -> Router {
    Router::new().route("/register", post(register_user))
}
