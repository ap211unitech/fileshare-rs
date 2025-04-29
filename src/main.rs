use axum::{body::Body, extract::Request, response::Redirect, routing::get, Extension, Router};
use config::{AppConfig, AppState};
use routes::{file::get_file_routes, health::get_health_routes, user::get_user_routes};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::FmtSubscriber;
use utils::tracing::Tracing;

mod config;
mod dtos;
mod error;
mod handler;
mod models;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    // Setting up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO) // error > warn > info > debug > trace
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to setup logging!");

    let app_config = AppConfig::load_config();
    let app_state = AppState::load_state().await;

    tracing::info!("Connected to database ✅");

    let router = Router::new()
        .route("/", get(Redirect::permanent("/health")))
        .nest("/health", get_health_routes())
        .nest("/user", get_user_routes())
        .nest("/file", get_file_routes())
        .layer(Extension(app_state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|_: &Request<Body>| tracing::info_span!("http"))
                .on_request(Tracing::on_request)
                .on_response(Tracing::on_response)
                .on_failure(Tracing::on_failure),
        );

    let listener = TcpListener::bind(app_config.server_url).await.unwrap();

    tracing::info!("Server started on: {} 🚀", listener.local_addr().unwrap());

    // Run server
    axum::serve(listener, router)
        .await
        .expect("Error serving application!");
}
