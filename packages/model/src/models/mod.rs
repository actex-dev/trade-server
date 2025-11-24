use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, Database, DbErr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod user;
pub mod admin;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timestamps {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Default for Timestamps {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

pub trait Model {
    fn id(&self) -> Uuid;
    fn timestamps(&self) -> &Timestamps;
    fn timestamps_mut(&mut self) -> &mut Timestamps;
}

pub trait SoftDelete: Model {
    fn is_deleted(&self) -> bool {
        <Self as Model>::timestamps(self).deleted_at.is_some()
    }

    fn soft_delete(&mut self) {
        <Self as Model>::timestamps_mut(self).deleted_at = Some(Utc::now());
    }

    fn restore(&mut self) {
        <Self as Model>::timestamps_mut(self).deleted_at = None;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum VisibilityStatus {
    Public,
    Private,
    Protected,
}

#[derive(Clone)]
pub struct Models {
    pub db: DatabaseConnection,
    pub user: user::repo::UserRepository,
    pub admin: admin::repo::AdminRepository,
}

impl Models {
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        let db = Database::connect(database_url).await?;
        Ok(Self {
            user: user::repo::UserRepository::new(db.clone()),
            admin: admin::repo::AdminRepository::new(db.clone()),
            db,
        })
    }
}

#[derive(Debug)]
pub enum ModelsError {
    NotFound(String),
    Duplicate(String),
    DatabaseError(String),
}

#[derive(Debug)]
pub enum ModelError {
    NotFound(String),
    Duplicate(String),
    DatabaseError(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ModelError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ModelError::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
            ModelError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for ModelError {}

impl std::fmt::Display for ModelsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ModelsError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ModelsError::Duplicate(msg) => write!(f, "Duplicate: {}", msg),
            ModelsError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for ModelsError {}
