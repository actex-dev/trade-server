use axum::{response::Response};
use axum::middleware::Next;
use axum::extract::Request;
use std::time::Instant;
use tracing::info;

pub async fn structured_logger(req: Request, next: Next) -> Result<Response, std::convert::Infallible> {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Capture request ID if present
    let request_id = req.extensions().get::<String>().cloned().unwrap_or_default();

    let res = next.run(req).await;
    let status = res.status().as_u16();
    let latency_ms = start.elapsed().as_millis();

    info!(
        request_id = %request_id,
        method = %method,
        path = %uri,
        status = %status,
        latency_ms = %latency_ms,
        "HTTP request"
    );

    Ok(res)
}