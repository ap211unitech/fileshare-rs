use axum::{routing::get, Router};

pub fn get_user_routes() -> Router {
    Router::new().route("/", get(|| async { "User is Healthy!" }))
}
