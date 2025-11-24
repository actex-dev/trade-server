use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc};

use super::{Admin};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "admins")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub email_address: String,
    pub password: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Admin {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            email_address: model.email_address,
            password: model.password,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
            deleted_at: model.deleted_at.map(|dt| dt.with_timezone(&Utc)),
        }
    }
}



impl From<Admin> for ActiveModel {
    fn from(admin: Admin) -> Self {
        Self {
            id: Set(admin.id),
            email_address: Set(admin.email_address),
            password: Set(admin.password),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            deleted_at: Set(None),
        }
    }
}