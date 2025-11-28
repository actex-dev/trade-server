use std::convert::Infallible;

use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
    middleware::Next,
    extract::Request,
};

use crate::shared::data::{AuthUser, state::AppState};
use crate::shared::data::ErrorResponse;

use repository::repositories::encryption::{EncryptionRepository, EncryptionRepositoryTrait, data::{Claims, Token, Sub}};
use std::sync::Arc;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use async_trait::async_trait;
// Convenience trait to convert to Response
use axum::response::IntoResponse;

fn unauthorized(message: &str) -> Response {
    let body = axum::Json(ErrorResponse::new(message.to_string()));
    (StatusCode::UNAUTHORIZED, body).into_response()
}

pub async fn require_user_auth(mut req: Request, next: Next) -> Result<Response, Infallible> {
    // Prefer EncryptionRepository from request extensions; fall back to AppState
    let encryption: Arc<EncryptionRepository> = if let Some(enc) = req.extensions().get::<Arc<EncryptionRepository>>() {
        enc.clone()
    } else if let Some(app_state) = req.extensions().get::<AppState>() {
        app_state.repository.encryption.clone()
    } else {
        return Ok(unauthorized("missing encryption repository"));
    };

    // Get Authorization header
    let headers: &HeaderMap = req.headers();
    let Some(auth_header_value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Ok(unauthorized("missing authorization header"));
    };

    let auth_str = match auth_header_value.to_str() {
        Ok(s) => s,
        Err(_) => return Ok(unauthorized("invalid authorization header")),
    };
    // Expect Bearer token
    let Some(token) = auth_str.strip_prefix("Bearer ") else {
        return Ok(unauthorized("invalid bearer token"));
    };

    // Normalize token: trim whitespace and surrounding quotes if present
    let token = token.trim();
    let token = token.trim_matches('"');

    // tracing::info!("token {}", token);
    // Decode user access token
    let claim = match encryption.decode_token(&token, Token::user_access_token()) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(msg = "invalid or expired token", err = ?err);
            return Ok(unauthorized("invalid or expired token"))
        },
    };

    // Decode Claims: handle both pasted JSON string and JSON value
    let claims: Claims = if let Some(s) = claim.as_str() {
        match serde_json::from_str::<Claims>(s) {
            Ok(c) => c,
            Err(err) => {
                tracing::error!(msg = "invalid token claims string", err = ?err);
                return Ok(unauthorized("invalid token claims"))
            }
        }
    } else {
        match serde_json::from_value::<Claims>(claim) {
            Ok(c) => c,
            Err(err) => {
                tracing::error!(msg = "invalid token claims value", err = ?err);
                return Ok(unauthorized("invalid token claims"))
            }
        }
    };

    let auth_user: AuthUser = match AuthUser::from_claims(claims) {
        Ok(u) => u,
        Err(err) => {
            tracing::error!(msg = "invalid token claims", err = ?err);
            return Ok(unauthorized("invalid token claims"))
        },
    };

    // Attach to request extensions for downstream handlers
    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

pub async fn require_refresh_auth(mut req: Request, next: Next) -> Result<Response, Infallible> {
    // Prefer EncryptionRepository from request extensions; fall back to AppState
    let encryption: Arc<EncryptionRepository> = if let Some(enc) = req.extensions().get::<Arc<EncryptionRepository>>() {
        enc.clone()
    } else if let Some(app_state) = req.extensions().get::<AppState>() {
        app_state.repository.encryption.clone()
    } else {
        return Ok(unauthorized("missing encryption repository"));
    };

    // Get Authorization header
    let headers: &HeaderMap = req.headers();
    let Some(auth_header_value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Ok(unauthorized("missing authorization header"));
    };

    let auth_str = match auth_header_value.to_str() {
        Ok(s) => s,
        Err(_) => return Ok(unauthorized("invalid authorization header")),
    };

    // Expect Bearer token
    let Some(token) = auth_str.strip_prefix("Bearer ") else {
        return Ok(unauthorized("invalid bearer token"));
    };

    // Normalize token: trim whitespace and surrounding quotes if present
    let token = token.trim();
    let token = token.trim_matches('"');

    // Decode refresh token
    let claim = match encryption.decode_token(&token, Token::user_refresh_token()) {
        Ok(v) => v,
        Err(_) => return Ok(unauthorized("invalid or expired token")),
    };

    // Parse Claims then extract AuthUser from sub
    let claims: Claims = match serde_json::from_value(claim) {
        Ok(c) => c,
        Err(_) => return Ok(unauthorized("invalid token claims")),
    };
    let auth_user: AuthUser = match &claims.sub {
        Sub::Text(s) => match serde_json::from_str::<AuthUser>(s) {
            Ok(u) => u,
            Err(_) => return Ok(unauthorized("invalid token claims")),
        },
        Sub::Json(v) => {
            if let Some(s) = v.as_str() {
                match serde_json::from_str::<AuthUser>(s) {
                    Ok(u) => u,
                    Err(_) => return Ok(unauthorized("invalid token claims")),
                }
            } else {
                match serde_json::from_value::<AuthUser>(v.clone()) {
                    Ok(u) => u,
                    Err(_) => return Ok(unauthorized("invalid token claims")),
                }
            }
        },
    };

    // Attach to request extensions for downstream handlers
    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

// Extractor-based middleware: validates user access token and injects AuthUser
#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Get Authorization header
        let Some(auth_header_value) = parts.headers.get(axum::http::header::AUTHORIZATION) else {
            return Err(unauthorized("missing authorization header"));
        };

        let auth_str = match auth_header_value.to_str() {
            Ok(s) => s,
            Err(_) => return Err(unauthorized("invalid authorization header")),
        };

        // Expect Bearer token
        let Some(token) = auth_str.strip_prefix("Bearer ") else {
            return Err(unauthorized("invalid bearer token"));
        };

        // Decode user access token using application state
        let encryption = &state.repository.encryption;
        let claim = match encryption.decode_token(&token, Token::user_access_token()) {
            Ok(v) => v,
            Err(_) => return Err(unauthorized("invalid or expired token")),
        };

        // Parse Claims then extract AuthUser from sub
        let claims: Claims = match serde_json::from_value(claim) {
            Ok(c) => c,
            Err(_) => return Err(unauthorized("invalid token claims")),
        };
        let auth_user: AuthUser = match &claims.sub {
            Sub::Text(s) => match serde_json::from_str::<AuthUser>(s) {
                Ok(u) => u,
                Err(_) => return Err(unauthorized("invalid token claims")),
            },
            Sub::Json(v) => {
                if let Some(s) = v.as_str() {
                    match serde_json::from_str::<AuthUser>(s) {
                        Ok(u) => u,
                        Err(_) => return Err(unauthorized("invalid token claims")),
                    }
                } else {
                    match serde_json::from_value::<AuthUser>(v.clone()) {
                        Ok(u) => u,
                        Err(_) => return Err(unauthorized("invalid token claims")),
                    }
                }
            },
        };

        Ok(auth_user)
    }
}