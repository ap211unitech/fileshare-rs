use axum::{middleware, routing::post, Router};

use crate::{handler::file::upload_file, utils::extractor::ExtractAuthAgent};

pub fn get_file_routes() -> Router {
    Router::new()
        .route("/upload", post(upload_file))
        // 🔒 Require ExtractAuthAgent on all routes
        .route_layer(middleware::from_extractor::<ExtractAuthAgent>())
}
