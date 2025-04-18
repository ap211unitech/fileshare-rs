use axum::{routing::get, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(|| async { "Healthy!" }));

    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    println!("Server started on: {} 🚀", listener.local_addr().unwrap());

    axum::serve(listener, router)
        .await
        .expect("Error serving application!");
}
