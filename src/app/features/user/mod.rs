use axum::Router;
pub mod auth;
pub mod profile;

use crate::app::shared::data::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/profile", profile::router())
}