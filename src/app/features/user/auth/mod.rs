use axum::{
    extract::{State, Json, Extension},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use model::models::user;
use crate::shared::{
    data::{ErrorResponse, SuccessResponse},
    middlewares::auth::require_refresh_auth,
    data::state::AppState,
};
use crate::shared::data::{AuthUser};

pub mod service;
pub mod password;

use service::{AuthError, AuthService};

/// Authentication controller that handles user authentication endpoints
pub struct AuthController;

impl AuthController {
    /// Creates a new AuthService instance from AppState
    fn create_auth_service(app_state: &AppState) -> AuthService {
        AuthService::new(
            app_state.model.user.clone(),
            (*app_state.repository.encryption).clone(),
        )
    }

    /// Handle user registration
    pub async fn sign_up(
        State(app_state): State<AppState>,
        Json(request): Json<user::RegisterRequest>,
    ) -> impl IntoResponse {
        let auth_service = Self::create_auth_service(&app_state);
        
        match auth_service.sign_up(request).await {
            Ok(response) => {
                (StatusCode::CREATED, Json(SuccessResponse::new(response))).into_response()
            }
            Err(AuthError::EmailAlreadyExists) => (
                StatusCode::CONFLICT,
                Json(ErrorResponse::new("Email address already exists".to_string())),
            ).into_response(),
            Err(AuthError::PasswordInvalid) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Password is invalid".to_string())),
            ).into_response(),
            Err(AuthError::DatabaseError(msg)) => {
                tracing::error!(error = %msg, "auth sign_up database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", msg))),
                )
                    .into_response()
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to create user".to_string())),
            ).into_response(),
        }
    }

    /// Handle user login
    pub async fn sign_in(
        State(app_state): State<AppState>,
        Json(request): Json<user::LoginRequest>,
    ) -> impl IntoResponse {
        let auth_service = Self::create_auth_service(&app_state);
        
        match auth_service.sign_in(request).await {
            Ok(response) => {
                (StatusCode::OK, Json(SuccessResponse::new(response))).into_response()
            }
            Err(AuthError::InvalidCredentials) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Invalid credentials".to_string())),
            ).into_response(),
            Err(AuthError::UserNotFound) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("User not found".to_string())),
            ).into_response(),
            Err(AuthError::DatabaseError(msg)) => {
                tracing::error!(error = %msg, "auth sign_in database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", msg))),
                )
                    .into_response()
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to sign in".to_string())),
            ).into_response(),
        }
    }

    /// Handle token refresh
    /// TODO: Implement proper JWT token extraction and validation
    pub async fn refresh_token(
        State(app_state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> impl IntoResponse {
        let auth_service = Self::create_auth_service(&app_state);

        match auth_service.refresh_token(auth_user).await {
            Ok(response) => (StatusCode::OK, Json(SuccessResponse::new(response))).into_response(),
            Err(AuthError::TokenCreationFailed) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to create token".to_string())),
            )
                .into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to refresh token".to_string())),
            )
                .into_response(),
        }
    }
}

/// Create the authentication router with all auth endpoints
pub fn router() -> Router<AppState> {
    let refresh_router = Router::new()
        .route("/refresh-token", post(AuthController::refresh_token))
        .layer(axum::middleware::from_fn(require_refresh_auth));

    Router::new()
        .route("/sign-up", post(AuthController::sign_up))
        .route("/sign-in", post(AuthController::sign_in))
        .merge(refresh_router)
        .nest("/password", password::router())
}