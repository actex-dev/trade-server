pub mod blockchain_client;
// pub mod token_data;
pub mod service;

use axum::Router;

pub fn router() -> Router {
    Router::new().route(
        "/:token_address",
        axum::routing::get(service::handle_token_websocket),
    )
}
