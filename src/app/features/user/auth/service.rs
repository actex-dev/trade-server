use uuid::Uuid;
use chrono::Utc;
use model::models::{user::repo::UserRepositoryTrait};
use model::models::user::{repo::UserRepository, model as user, entity as user_entity};
use repository::repositories::{encryption::{EncryptionRepository, EncryptionRepositoryTrait, data::Token}};
use crate::app::shared::data::{AuthUser};

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    EmailAlreadyExists,
    PasswordInvalid,
    TokenCreationFailed,
    DatabaseError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::EmailAlreadyExists => write!(f, "Email already exists"),
            AuthError::PasswordInvalid => write!(f, "Password is invalid"),
            AuthError::TokenCreationFailed => write!(f, "Failed to create token"),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    encryption_repo: EncryptionRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, encryption_repo: EncryptionRepository) -> Self {
        Self {
            user_repo,
            encryption_repo,
        }
    }

    pub async fn sign_up(&self, request: user::RegisterRequest) -> Result<user::AuthUserResponse, AuthError> {
        // Hash password
        let hash_password = self.encryption_repo.hash_password(&request.password)
            .map_err(|_| AuthError::PasswordInvalid)?;

        // Check if user already exists
        let user_exist = match self.user_repo.get_by_email(&request.email_address.to_lowercase())
            .await {
            Ok(user) => Ok(user),
            Err(e) => Err(AuthError::DatabaseError(e.to_string())),
        };

        if user_exist.is_ok() {
            return Err(AuthError::EmailAlreadyExists);
        }

        // Create new user
        let new_user = user_entity::Model {
            id: Uuid::new_v4(),
            personal_first_name: request.first_name.clone(),
            personal_second_name: request.second_name.clone(),
            personal_email_address: request.email_address.clone().to_lowercase(),
            personal_profile_image: None,   
            personal_username: None,
            password: hash_password,
            peripheral_authentication_code: None,
            peripheral_authentication_token: None,
            peripheral_timeout: None,
            peripheral_is_banned: false,
            peripheral_is_verified: false,
            verification_code: String::new(),
            verification_timeout: None,
            setting_custom_setting_default_theme: None,
            setting_custom_setting_is_accepting_request: false,
            setting_subscription_price_id: None,
            setting_subscription_product_id: None,
            setting_subscription_status: "BASIC".to_string(),
            setting_subscription_start_date: None,
            setting_subscription_end_date: None,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
            deleted_at: None,
        };

        // Save user
        let created_user = match self.user_repo.create(new_user).await {
            Ok(user) => Ok(user),
            Err(e) => Err(AuthError::DatabaseError(e.to_string())),
        }?;

        // Create tokens
        let auth_user = AuthUser::from_user(created_user);

        let access_token = self.encryption_repo.create_token(auth_user.clone(), Token::user_access_token())
            .map_err(|_| AuthError::TokenCreationFailed)?;
        
        let refresh_token = self.encryption_repo.create_token(auth_user.clone(), Token::user_refresh_token())
            .map_err(|_| AuthError::TokenCreationFailed)?;

        Ok(user::AuthUserResponse {
            id: auth_user.id.to_string(),
            access_token,
            refresh_token,
        })
    }

    pub async fn sign_in(&self, request: user::LoginRequest) -> Result<user::AuthUserResponse, AuthError> {
        // Get user by email
        let user = self.user_repo.get_by_email(&request.email_address.to_lowercase())
            .await
            .map_err(|_| AuthError::UserNotFound)?;

        // Verify password
        let is_valid = self.encryption_repo.verify_password(&user.password, &request.password)
            .map_err(|_| AuthError::PasswordInvalid)?;
        
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Create tokens
        let auth_user = AuthUser::from_user(user);

        let access_token = self.encryption_repo.create_token(auth_user.clone(), Token::user_access_token())
            .map_err(|_| AuthError::TokenCreationFailed)?;
        
        let refresh_token = self.encryption_repo.create_token(auth_user.clone(), Token::user_refresh_token())
            .map_err(|_| AuthError::TokenCreationFailed)?;

        Ok(user::AuthUserResponse {
            id: auth_user.id.to_string(),
            access_token,
            refresh_token,
        })
    }

    pub async fn refresh_token(&self, auth_user: AuthUser) -> Result<user::AuthUserResponse, AuthError> {
        let access_token = self.encryption_repo.create_token(auth_user.clone(), Token::user_access_token())
            .map_err(|_| AuthError::TokenCreationFailed)?;
        let refresh_token = self.encryption_repo.create_token(auth_user.clone(), Token::user_refresh_token())
            .map_err(|_| AuthError::TokenCreationFailed)?;

        Ok(user::AuthUserResponse {
            id: auth_user.id.to_string(),
            access_token,
            refresh_token, // Empty for refresh token endpoint
        })
    }
}

