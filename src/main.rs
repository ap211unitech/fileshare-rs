use axum::{response::Redirect, routing::get, Router};
use config::{AppConfig, AppState};
use routes::{health::get_health_routes, user::get_user_routes};
use tokio::net::TcpListener;

mod config;
mod handler;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    let app_config = AppConfig::load_config();
    let app_state = AppState::load_state().await;

    println!("Connected to database ✅");

    let router = Router::new()
        .route("/", get(Redirect::permanent("/health")))
        .nest("/health", get_health_routes())
        .nest("/user", get_user_routes());

    let listener = TcpListener::bind(app_config.server_url).await.unwrap();

    println!("Server started on: {} 🚀", listener.local_addr().unwrap());

    axum::serve(listener, router)
        .await
        .expect("Error serving application!");
}
