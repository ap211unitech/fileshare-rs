use axum::{middleware, response::IntoResponse, routing::get, Router};

use crate::utils::extractor::ExtractAuthAgent;

pub fn get_file_routes() -> Router {
    Router::new()
        .route("/", get(method_router))
        // 🔒 Require ExtractAuthAgent on all routes
        .route_layer(middleware::from_extractor::<ExtractAuthAgent>())
}

async fn method_router(agent: ExtractAuthAgent) -> impl IntoResponse {
    println!("{:?}", agent);
    "file secret data".to_string()
}
