pub mod state;

use repository::repositories::encryption::data::{Claims, Sub};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModelStatus {
    Success,
    Error,
    Pending,
    Processing,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    pub status: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuccessResponse<T> {
    pub status: bool,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
    pub status: ModelStatus,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { status: true, data }
    }
}

impl ErrorResponse {
    pub fn new(message: String) -> Self {
        Self { status: false, message }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub first_name: String,
    pub email_address: String,
}

impl AuthUser {
    pub fn from_user(user: model::models::user::Model) -> Self {
        Self {
            id: user.id,
            first_name: user.personal_first_name,
            email_address: user.personal_email_address,
        }
    }

    pub fn from_claims(claims: Claims) -> Result<AuthUser, String> {
        return match &claims.sub {
            Sub::Text(s) => match serde_json::from_str::<AuthUser>(s) {
                Ok(auth_user) => Ok(auth_user),
                Err(err) => {
                    tracing::error!(msg = "invalid string token claims", err = ?err);
                    return Err("invalid token claims".to_string())
                },
            },
            Sub::Json(v) => {
                if let Some(s) = v.as_str() {
                    match serde_json::from_str::<AuthUser>(s) {
                        Ok(auth_user) => Ok(auth_user),
                        Err(err) => {
                            tracing::error!(msg = "invalid string token claims", err = ?err);
                            return Err("invalid token claims".to_string())
                        },
                    }
                } else {
                    match serde_json::from_value::<AuthUser>(v.clone()) {
                        Ok(auth_user) => Ok(auth_user),
                        Err(err) => {
                            tracing::error!(msg = "invalid token claims", err = ?err);
                            return Err("invalid token claims".to_string())
                        },
                    }
                }
            },
        };
    }
}