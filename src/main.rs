use axum::{routing::get, Router};
use config::{AppConfig, AppState};
use tokio::net::TcpListener;

mod config;

#[tokio::main]
async fn main() {
    let app_config = AppConfig::load_config();
    let app_state = AppState::load_state();

    println!("Connected to database ✅");

    let router = Router::new().route("/", get(|| async { "Healthy!" }));

    let listener = TcpListener::bind(app_config.server_url).await.unwrap();

    println!("Server started on: {} 🚀", listener.local_addr().unwrap());

    axum::serve(listener, router)
        .await
        .expect("Error serving application!");
}
