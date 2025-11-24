use axum::http::{HeaderValue, HeaderName};
use axum::{response::Response};
use axum::middleware::Next;
use axum::extract::Request;
use uuid::Uuid;

pub async fn set_request_id(mut req: Request, next: Next) -> Result<Response, std::convert::Infallible> {
    let request_id = Uuid::new_v4().to_string();

    // Insert into request headers
    let header_name = HeaderName::from_static("x-request-id");
    req.headers_mut().insert(header_name.clone(), HeaderValue::from_str(&request_id).unwrap());

    // Add to extensions for downstream access
    req.extensions_mut().insert(request_id.clone());

    let mut res = next.run(req).await;

    // Propagate to response headers
    res.headers_mut().insert(header_name, HeaderValue::from_str(&request_id).unwrap());

    Ok(res)
}