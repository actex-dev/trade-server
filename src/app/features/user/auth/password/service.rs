use chrono::{Duration, Utc};
use uuid::Uuid;

use model::models::user::{self as user, repo::UserRepositoryTrait};
use model::models::user::repo::UserRepository;
use repository::repositories::encryption::{EncryptionRepository, EncryptionRepositoryTrait, data::Token};
use crate::shared::data::AuthUser;

#[derive(Debug)]
pub enum PasswordError {
    UserNotFound,
    CodeExpired,
    InvalidCode,
    PasswordMismatch,
    TokenCreationFailed,
    DatabaseError(String),
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PasswordError::UserNotFound => write!(f, "User not found"),
            PasswordError::CodeExpired => write!(f, "Code expired"),
            PasswordError::InvalidCode => write!(f, "Invalid code"),
            PasswordError::PasswordMismatch => write!(f, "Passwords do not match"),
            PasswordError::TokenCreationFailed => write!(f, "Failed to create token"),
            PasswordError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for PasswordError {}

#[derive(Clone)]
pub struct PasswordService {
    user_repo: UserRepository,
    encryption_repo: EncryptionRepository,
}

impl PasswordService {
    pub fn new(user_repo: UserRepository, encryption_repo: EncryptionRepository) -> Self {
        Self { user_repo, encryption_repo }
    }

    // Send reset code to the email address, storing it and timeout on the user
    pub async fn send_reset_code(
        &self,
        request: user::SendResetCodeRequest,
    ) -> Result<user::PasswordAuthResponse, PasswordError> {
        let mut model = self
            .user_repo
            .get_by_email(&request.email_address.to_lowercase())
            .await
            .map_err(|_| PasswordError::UserNotFound)?;

        let code = self.encryption_repo.create_code(6);
        model.peripheral_authentication_code = Some(code);
        model.peripheral_timeout = Some(Utc::now().into());

        let updated = self
            .user_repo
            .update(model)
            .await
            .map_err(|e| PasswordError::DatabaseError(e.to_string()))?;

        Ok(user::PasswordAuthResponse {
            email_address: updated.personal_email_address,
            message: "code has been sent to this email".to_string(),
        })
    }

    // Verify code and return a short-lived token
    pub async fn verify_code(
        &self,
        req: user::VerifyResetCodeRequest,
    ) -> Result<user::VerifyCodeResponse, PasswordError> {
        let model = self
            .user_repo
            .get_by_email(&req.email_address.to_lowercase())
            .await
            .map_err(|_| PasswordError::UserNotFound)?;

        // Check code matches
        match &model.peripheral_authentication_code {
            Some(stored) if stored == &req.auth_code => {}
            _ => return Err(PasswordError::InvalidCode),
        }

        // Check not expired (older than 7 days considered expired)
        let timeout_utc = model
            .peripheral_timeout
            .map(|t| chrono::DateTime::<Utc>::from(t))
            .ok_or(PasswordError::CodeExpired)?;

        if Utc::now() - timeout_utc > Duration::days(7) {
            return Err(PasswordError::CodeExpired);
        }

        // Build auth payload and create token
        let auth_user = AuthUser {
            id: model.id,
            first_name: model.personal_first_name,
            email_address: model.personal_email_address,
        };

        let token = self
            .encryption_repo
            .create_token(auth_user, Token::user_refresh_token())
            .map_err(|_| PasswordError::TokenCreationFailed)?;

        Ok(user::VerifyCodeResponse {
            token,
            message: "code has been verify successful".to_string(),
        })
    }

    // Reset password for the authenticated user
    pub async fn reset_password(
        &self,
        auth_user_id: Uuid,
        req: user::ResetPasswordRequest,
    ) -> Result<user::PasswordAuthResponse, PasswordError> {
        if req.password != req.confirm_password {
            return Err(PasswordError::PasswordMismatch);
        }

        let mut model = self
            .user_repo
            .get_by_id(auth_user_id)
            .await
            .map_err(|_| PasswordError::UserNotFound)?;

        // Check not expired (older than 7 days considered expired)
        let timeout_utc = model
            .peripheral_timeout
            .map(|t| chrono::DateTime::<Utc>::from(t))
            .ok_or(PasswordError::CodeExpired)?;
        if Utc::now() - timeout_utc > Duration::days(7) {
            return Err(PasswordError::CodeExpired);
        }

        // Hash and update password
        let hashed = self
            .encryption_repo
            .hash_password(&req.password)
            .map_err(|_| PasswordError::DatabaseError("password hash failed".to_string()))?;

        model.password = hashed;

        let updated = self
            .user_repo
            .update(model)
            .await
            .map_err(|e| PasswordError::DatabaseError(e.to_string()))?;

        Ok(user::PasswordAuthResponse {
            email_address: updated.personal_email_address,
            message: "code has been sent to this email".to_string(),
        })
    }
}