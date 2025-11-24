use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait};
use async_trait::async_trait;
use uuid::Uuid;
use crate::models::admin::{self, entity::Entity as AdminEntity, entity::Model as AdminModel};

#[derive(Debug)]
pub enum AdminRepositoryError {
    NotFound(String),
    Duplicate(String),
    DatabaseError(String),
}

impl std::fmt::Display for AdminRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AdminRepositoryError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AdminRepositoryError::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
            AdminRepositoryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AdminRepositoryError {}

#[async_trait]
pub trait AdminRepositoryTrait {
    async fn create(&self, admin: AdminModel) -> Result<AdminModel, AdminRepositoryError>;
    async fn get_by_id(&self, id: Uuid) -> Result<AdminModel, AdminRepositoryError>;
    async fn get_by_email(&self, email: &str) -> Result<AdminModel, AdminRepositoryError>;
    async fn update(&self, admin: AdminModel) -> Result<AdminModel, AdminRepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), AdminRepositoryError>;
    async fn list_all(&self) -> Result<Vec<AdminModel>, AdminRepositoryError>;
}

#[derive(Clone)]
pub struct AdminRepository {
    db: DatabaseConnection,
}

impl AdminRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AdminRepositoryTrait for AdminRepository {
    async fn create(&self, admin: AdminModel) -> Result<AdminModel, AdminRepositoryError> {
        let active_model: admin::entity::ActiveModel = admin.clone().into();

        match active_model.insert(&self.db).await {
            Ok(inserted) => Ok(inserted),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("duplicate") || error_msg.contains("unique") {
                    Err(AdminRepositoryError::Duplicate("Admin with this email already exists".to_string()))
                } else {
                    Err(AdminRepositoryError::DatabaseError(error_msg))
                }
            }
        }
    }

    async fn get_by_id(&self, id: Uuid) -> Result<AdminModel, AdminRepositoryError> {
        match AdminEntity::find_by_id(id).one(&self.db).await {
            Ok(Some(admin)) => Ok(admin),
            Ok(None) => Err(AdminRepositoryError::NotFound(format!("Admin with id {} not found", id))),
            Err(e) => Err(AdminRepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn get_by_email(&self, email: &str) -> Result<AdminModel, AdminRepositoryError> {
        match AdminEntity::find()
            .filter(admin::entity::Column::EmailAddress.eq(email))
            .one(&self.db)
            .await
        {
            Ok(Some(admin)) => Ok(admin),
            Ok(None) => Err(AdminRepositoryError::NotFound(format!("Admin with email {} not found", email))),
            Err(e) => Err(AdminRepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn update(&self, admin: AdminModel) -> Result<AdminModel, AdminRepositoryError> {
        let active_model: admin::entity::ActiveModel = admin.clone().into();

        match active_model.update(&self.db).await {
            Ok(updated) => Ok(updated),
            Err(e) => Err(AdminRepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), AdminRepositoryError> {
        match AdminEntity::delete_by_id(id).exec(&self.db).await {
            Ok(_) => Ok(()),
            Err(e) => Err(AdminRepositoryError::DatabaseError(e.to_string())),
        }
    }

    async fn list_all(&self) -> Result<Vec<AdminModel>, AdminRepositoryError> {
        match AdminEntity::find().all(&self.db).await {
            Ok(admins) => Ok(admins),
            Err(e) => Err(AdminRepositoryError::DatabaseError(e.to_string())),
        }
    }
}