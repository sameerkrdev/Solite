use axum::{Router, routing::get};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello World" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Starting server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listerner) => listerner,
        Err(err) => {
            panic!("Failed to bind TCP listener: {}", err);
        }
    };

    if let Err(err) = axum::serve(listener, app).await {
        panic!("server error: {}", err)
    };
}
