use axum::http::{Method, header};
use axum::Router;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

pub mod features;
pub mod shared;

async fn health_check() -> &'static str {
    "OK - Dex WebSocket Proxy"
}

#[tokio::main]
async fn main() {
    let _ = dotenv();
    
    // Initialize simple logger for dex binary
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", axum::routing::get(health_check))
        .nest("/api", features::router())
        .layer(cors);

    let address = SocketAddr::from(([127, 0, 0, 1], 8001));

    let tcp_listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind address");

    // Log active server port
    tracing::info!("Dex WebSocket Proxy running on port: {}", address.port());

    axum::serve(tcp_listener, app)
        .await
        .expect("Failed to start server");
}
