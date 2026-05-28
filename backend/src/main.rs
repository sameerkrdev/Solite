use std::net::SocketAddr;

mod api;
mod utils;
mod wallets;

use api::router;

#[tokio::main]
async fn main() {
    let app = router::build_router();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Starting server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            panic!("Failed to bind TCP listener: {}", err);
        }
    };

    if let Err(err) = axum::serve(listener, app).await {
        panic!("server error: {}", err)
    };
}
