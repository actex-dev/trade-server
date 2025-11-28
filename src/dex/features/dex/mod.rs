pub mod bsc;

use axum::Router;

pub fn router() -> Router {
    Router::new().nest("/bsc", bsc::router())
}
