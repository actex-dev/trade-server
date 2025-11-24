use axum::Router;
// pub mod admin;
pub mod user;

use axum::middleware;
use crate::app::shared::middlewares::{logging, recovery, request_id};

use crate::app::shared::data::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/user", user::router())
        .layer(middleware::from_fn(recovery::recover))
        .layer(middleware::from_fn(request_id::set_request_id))
        .layer(middleware::from_fn(logging::structured_logger))
}