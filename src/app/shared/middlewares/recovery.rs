use axum::{http::StatusCode, response::{Response}};
use axum::middleware::Next;
use axum::extract::Request;
use tracing::error;

pub async fn recover(req: Request, next: Next) -> Result<Response, std::convert::Infallible> {
    // Note: Panics in handlers cannot be safely caught here due to non-UnwindSafe next.
    // We still log unexpected conditions if needed.
    let res = next.run(req).await;
    if res.status() == StatusCode::INTERNAL_SERVER_ERROR {
        error!("Internal server error while handling request");
    }
    Ok(res)
}