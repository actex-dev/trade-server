use chrono::Utc;
use uuid::Uuid;

use model::models::user::{self as user, repo::UserRepositoryTrait};
use model::models::user::repo::UserRepository;

#[derive(Debug)]
pub enum ProfileError {
    NotFound(String),
    Duplicate(String),
    DatabaseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProfileError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ProfileError::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
            ProfileError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ProfileError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ProfileError {}

#[derive(Clone)]
pub struct ProfileService {
    user_repo: UserRepository,
}

impl ProfileService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<user::SecureUserResponse, ProfileError> {
        tracing::error!("profile get_me");
        let entity = self
            .user_repo
            .get_by_id(user_id)
            .await
            .map_err(|e| match e {
                model::models::user::repo::UserRepositoryError::NotFound(msg) => ProfileError::NotFound(msg),
                model::models::user::repo::UserRepositoryError::Duplicate(msg) => ProfileError::Duplicate(msg),
                model::models::user::repo::UserRepositoryError::DatabaseError(msg) => ProfileError::DatabaseError(msg),
            })?;

        let domain_user: user::User = entity.into();

        Ok(user::SecureUserResponse::from(domain_user))
    }

    pub async fn update_personal(
        &self,
        user_id: Uuid,
        req: user::UpdatePersonal,
    ) -> Result<user::SecureUserResponse, ProfileError> {
        // Basic validation
        if req.first_name.trim().is_empty() || req.second_name.trim().is_empty() {
            return Err(ProfileError::ValidationError("first_name and second_name are required".to_string()));
        }
        if req.email_address.trim().is_empty() {
            return Err(ProfileError::ValidationError("email_address is required".to_string()));
        }

        let mut model = self
            .user_repo
            .get_by_id(user_id)
            .await
            .map_err(|e| match e {
                model::models::user::repo::UserRepositoryError::NotFound(msg) => ProfileError::NotFound(msg),
                model::models::user::repo::UserRepositoryError::Duplicate(msg) => ProfileError::Duplicate(msg),
                model::models::user::repo::UserRepositoryError::DatabaseError(msg) => ProfileError::DatabaseError(msg),
            })?;

        // Apply changes
        model.personal_first_name = req.first_name;
        model.personal_second_name = req.second_name;
        model.personal_email_address = req.email_address.to_lowercase();
        model.personal_profile_image = req.profile_image;
        model.personal_username = req.username;
        model.updated_at = Utc::now().into();

        // Persist
        let updated = self
            .user_repo
            .update(model)
            .await
            .map_err(|e| match e {
                // Map duplicate email constraint if any
                model::models::user::repo::UserRepositoryError::DatabaseError(msg) => {
                    if msg.to_lowercase().contains("duplicate") || msg.to_lowercase().contains("unique") {
                        ProfileError::Duplicate("email address already exists".to_string())
                    } else {
                        ProfileError::DatabaseError(msg)
                    }
                }
                model::models::user::repo::UserRepositoryError::NotFound(msg) => ProfileError::NotFound(msg),
                model::models::user::repo::UserRepositoryError::Duplicate(msg) => ProfileError::Duplicate(msg),
            })?;

        let domain_user: user::User = updated.into();
        Ok(user::SecureUserResponse::from(domain_user))
    }
}