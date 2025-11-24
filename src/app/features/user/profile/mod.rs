use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};

use crate::app::shared::{
    data::{AuthUser, ErrorResponse, SuccessResponse},
    middlewares::auth::require_user_auth,
    data::state::AppState,
};

use model::models::user;

mod service;
use service::{ProfileError, ProfileService};

pub struct ProfileController;

impl ProfileController {
    fn create_service(app_state: &AppState) -> ProfileService {
        ProfileService::new(
            app_state.model.user.clone(),
        )
    }

    pub async fn get_me(
        State(app_state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> impl IntoResponse {
        let service = Self::create_service(&app_state);
        match service.get_profile(auth_user.id).await {
            Ok(resp) => (StatusCode::OK, Json(SuccessResponse::new(resp))).into_response(),
            Err(ProfileError::NotFound(msg)) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(msg)),
            )
                .into_response(),
            Err(ProfileError::DatabaseError(msg)) => {
                tracing::error!(error = %msg, "profile get_me database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", msg))),
                )
                    .into_response()
            }
            Err(ProfileError::Duplicate(msg)) | Err(ProfileError::ValidationError(msg)) => {
                tracing::error!(error = %msg, "profile get_me database error");
                (
                StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new(msg)),
                )
                    .into_response()
            }
        }
    }

    pub async fn update_me(
        State(app_state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(req): Json<user::UpdatePersonal>,
    ) -> impl IntoResponse {
        let service = Self::create_service(&app_state);
        match service.update_personal(auth_user.id, req).await {
            Ok(resp) => (StatusCode::OK, Json(SuccessResponse::new(resp))).into_response(),
            Err(ProfileError::NotFound(msg)) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(msg)),
            )
                .into_response(),
            Err(ProfileError::Duplicate(msg)) => (
                StatusCode::CONFLICT,
                Json(ErrorResponse::new(msg)),
            )
                .into_response(),
            Err(ProfileError::ValidationError(msg)) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(msg)),
            )
                .into_response(),
            Err(ProfileError::DatabaseError(msg)) => {
                tracing::error!(error = %msg, "profile update_me database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Database error: {}", msg))),
                )
                    .into_response()
            }
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(ProfileController::get_me))
        .route("/", put(ProfileController::update_me))
        // Apply function-based auth middleware which reads AppState from request extensions
        .layer(axum::middleware::from_fn(require_user_auth))
}