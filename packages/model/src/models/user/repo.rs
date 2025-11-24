use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait};
use async_trait::async_trait;
use uuid::Uuid;
use crate::models::user::{self, Entity as UserEntity, Model as UserModel};

#[derive(Debug)]
pub enum UserRepositoryError {
    NotFound(String),
    Duplicate(String),
    DatabaseError(String),
}

impl std::fmt::Display for UserRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserRepositoryError::NotFound(msg) => write!(f, "Not found: {}", msg),
            UserRepositoryError::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
            UserRepositoryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for UserRepositoryError {}

#[async_trait]
pub trait UserRepositoryTrait {
    async fn create(&self, user: UserModel) -> Result<UserModel, UserRepositoryError>;
    async fn get_by_id(&self, id: Uuid) -> Result<UserModel, UserRepositoryError>;
    async fn get_by_email(&self, email: &str) -> Result<UserModel, UserRepositoryError>;
    async fn update(&self, user: UserModel) -> Result<UserModel, UserRepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), UserRepositoryError>;
}

#[derive(Clone)]
pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn create(&self, user: UserModel) -> Result<UserModel, UserRepositoryError> {
        let active_model: user::entity::ActiveModel = user.clone().into();
        match active_model.insert(&self.db).await {
            Ok(inserted) => Ok(inserted),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("duplicate") || error_msg.contains("unique") {
                    Err(UserRepositoryError::Duplicate("Email address already exists".to_string()))
                } else {
                    Err(UserRepositoryError::DatabaseError(error_msg))
                }
            }
        }
    }

    async fn get_by_id(&self, id: Uuid) -> Result<UserModel, UserRepositoryError> {
        match UserEntity::find_by_id(id).one(&self.db).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(UserRepositoryError::NotFound(format!("User with id {} not found", id))),
            Err(e) => Err(UserRepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn get_by_email(&self, email: &str) -> Result<UserModel, UserRepositoryError> {
        match UserEntity::find()
            .filter(user::entity::Column::PersonalEmailAddress.eq(email))
            .one(&self.db)
            .await
        {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(UserRepositoryError::NotFound(format!("User with email {} not found", email))),
            Err(e) => Err(UserRepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn update(&self, user: UserModel) -> Result<UserModel, UserRepositoryError> {
        let active_model: user::entity::ActiveModel = user.clone().into();
        match active_model.update(&self.db).await {
            Ok(updated) => Ok(updated),
            Err(e) => Err(UserRepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), UserRepositoryError> {
        match UserEntity::delete_by_id(id).exec(&self.db).await {
            Ok(_) => Ok(()),
            Err(e) => Err(UserRepositoryError::DatabaseError(e.to_string())),
        }
    }
}

