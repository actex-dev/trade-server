use axum::Router;
pub mod dex;

pub fn router() -> Router {
    Router::new().nest("/dex", dex::router())
}
