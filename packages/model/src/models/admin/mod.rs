use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::shared::PaginatedResponse;

pub mod entity;
pub mod repo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin {
    pub id: Uuid,
    pub email_address: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Unified paginated response alias
pub type AdminsPage = PaginatedResponse<Admin>;

