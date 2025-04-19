use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    message: String,
}

pub fn get_health_routes() -> Router {
    Router::new().route("/", get(handler))
}

async fn handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            message: "Server is healthy!".to_string(),
        }),
    )
}
