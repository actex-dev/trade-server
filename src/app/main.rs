use shared::data::state::AppState;
use shared::utils::config::AppConfig;
use shared::utils::logger;
use axum::http::{Method, header};
use axum::{Extension, Router};
use dotenvy::dotenv;
use model::migration::{Migrator, MigratorTrait};
use model::models::Models;
use repository::repositories::Repositories;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

pub mod features;
pub mod shared;

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    let _ = dotenv();
    // Initialize global logger
    logger::init();
    let cfg = AppConfig::from_env();
    let models = match Models::new(&cfg.database_url).await {
        Ok(m) => m,
        Err(e) => {
            tracing::info!("Failed to connect to the database: {}", e);
            return;
        }
    };

    if let Err(e) = Migrator::up(&models.db, None).await {
        tracing::info!("Failed to run migrations: {}", e);
        return;
    }
    let repositories = Repositories::new();

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
        .nest("/api/", features::router())
        .layer(Extension(repositories.encryption.clone()))
        .with_state(AppState::new(repositories, models))
        .layer(cors);

    let address = SocketAddr::from(([127, 0, 0, 1], 8000));

    let tcp_listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind address");

    // Log active server port
    tracing::info!("running on port: {}", address.port());

    axum::serve(tcp_listener, app)
        .await
        .expect("Failed to start server");
}
