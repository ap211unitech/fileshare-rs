use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handler::file::{download_file, upload_file, user_files},
    utils::extractor::ExtractAuthAgent,
};

pub fn get_file_routes() -> Router {
    // Protected routes
    let protected_routes = Router::new()
        .route("/upload", post(upload_file))
        .route("/user-files", get(user_files))
        .route_layer(middleware::from_extractor::<ExtractAuthAgent>());

    // Public routes
    let public_routes = Router::new().route("/download", post(download_file));

    // Combine both
    protected_routes.merge(public_routes)
}
