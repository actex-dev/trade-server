use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};

use crate::shared::{
    data::{AuthUser, ErrorResponse, SuccessResponse},
    middlewares::auth::require_user_auth,
    data::state::AppState,
};
use model::models::user;
use super::password::service::{PasswordService, PasswordError};

pub mod service;

pub struct PasswordController;

impl PasswordController {
    fn create_service(app_state: &AppState) -> PasswordService {
        PasswordService::new(
            app_state.model.user.clone(),
            (*app_state.repository.encryption).clone(),
        )
    }

    pub async fn send_reset_code(
        State(app_state): State<AppState>,
        Json(request): Json<user::SendResetCodeRequest>,
    ) -> impl IntoResponse {
        let service = Self::create_service(&app_state);
        match service.send_reset_code(request).await {
            Ok(resp) => (StatusCode::OK, Json(SuccessResponse::new(resp))).into_response(),
            Err(PasswordError::UserNotFound) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("email address was not found".to_string())),
            )
                .into_response(),
            Err(PasswordError::DatabaseError(msg)) => {
                tracing::error!(error = %msg, "password send_reset_code database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", msg))),
                )
                    .into_response()
            }
            Err(_) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("unable to send verification code".to_string())),
            )
                .into_response(),
        }
    }

    pub async fn verify_code(
        State(app_state): State<AppState>,
        Json(request): Json<user::VerifyResetCodeRequest>,
    ) -> impl IntoResponse {
        let service = Self::create_service(&app_state);
        match service.verify_code(request).await {
            Ok(resp) => (StatusCode::OK, Json(SuccessResponse::new(resp))).into_response(),
            Err(PasswordError::UserNotFound) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("email is not registered with us".to_string())),
            )
                .into_response(),
            Err(PasswordError::InvalidCode) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid code".to_string())),
            )
                .into_response(),
            Err(PasswordError::CodeExpired) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("code expired".to_string())),
            )
                .into_response(),
            Err(PasswordError::TokenCreationFailed) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("unable to verify code".to_string())),
            )
                .into_response(),
            Err(PasswordError::DatabaseError(msg)) => {
                tracing::error!(error = %msg, "password verify_code database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", msg))),
                )
                    .into_response()
            }
            Err(_) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("unable to verify code".to_string())),
            )
                .into_response(),
        }
    }

    pub async fn reset_password(
        State(app_state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<user::ResetPasswordRequest>,
    ) -> impl IntoResponse {
        let service = Self::create_service(&app_state);
        match service.reset_password(auth_user.id, request).await {
            Ok(resp) => (StatusCode::OK, Json(SuccessResponse::new(resp))).into_response(),
            Err(PasswordError::PasswordMismatch) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("password are not the same".to_string())),
            )
                .into_response(),
            Err(PasswordError::CodeExpired) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("code expired".to_string())),
            )
                .into_response(),
            Err(PasswordError::UserNotFound) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("email is not registered with us".to_string())),
            )
                .into_response(),
            Err(PasswordError::DatabaseError(msg)) => {
                tracing::error!(error = %msg, "password reset_password database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", msg))),
                )
                    .into_response()
            }
            Err(_) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("unable to reset password".to_string())),
            )
                .into_response(),
        }
    }
}

pub fn router() -> Router<AppState> {
    let public = Router::new()
        .route("/send-reset-code", post(PasswordController::send_reset_code))
        .route("/verify-reset-code", post(PasswordController::verify_code));

    let protected = Router::new()
        .route("/reset-password", post(PasswordController::reset_password))
        .layer(axum::middleware::from_fn(require_user_auth));

    Router::new().nest("/", public).nest("/", protected)
}