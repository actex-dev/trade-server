use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::Timestamps;
use super::model::{User, Personal, Peripheral, Verification, Setting, CustomSetting, Subscription, SubscriptionStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    
    // Personal information
    pub personal_first_name: String,
    pub personal_second_name: String,
    #[sea_orm(unique)]
    pub personal_email_address: String,
    pub personal_profile_image: Option<String>,
    pub personal_username: Option<String>,
    
    // Password (never exposed)
    pub password: String,
    
    // Peripheral information
    pub peripheral_authentication_code: Option<String>,
    pub peripheral_authentication_token: Option<String>,
    pub peripheral_timeout: Option<DateTimeWithTimeZone>,
    pub peripheral_is_banned: bool,
    pub peripheral_is_verified: bool,
    
    // Verification
    pub verification_code: String,
    pub verification_timeout: Option<i64>,
    
    // Settings
    pub setting_custom_setting_default_theme: Option<String>,
    pub setting_custom_setting_is_accepting_request: bool,
    pub setting_subscription_price_id: Option<String>,
    pub setting_subscription_product_id: Option<String>,
    pub setting_subscription_status: String,
    pub setting_subscription_start_date: Option<DateTimeWithTimeZone>,
    pub setting_subscription_end_date: Option<DateTimeWithTimeZone>,
    
    // Timestamps
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for User {
    fn from(model: Model) -> Self {
        let timestamps = Timestamps {
            created_at: DateTime::<Utc>::from(model.created_at),
            updated_at: DateTime::<Utc>::from(model.updated_at),
            deleted_at: model.deleted_at.map(DateTime::<Utc>::from),
        };

        Self {
            id: model.id,
            personal: Personal {
                first_name: model.personal_first_name,
                second_name: model.personal_second_name,
                email_address: model.personal_email_address,
                profile_image: model.personal_profile_image,
                username: model.personal_username,
            },
            password: model.password,
            peripheral: Peripheral {
                authentication_code: model.peripheral_authentication_code,
                authentication_token: model.peripheral_authentication_token,
                timeout: model.peripheral_timeout.map(DateTime::<Utc>::from),
                is_banned: model.peripheral_is_banned,
                is_verified: model.peripheral_is_verified,
            },
            verification: Verification {
                code: model.verification_code,
                timeout: model
                    .verification_timeout
                    .and_then(|ts| chrono::DateTime::<Utc>::from_timestamp(ts, 0)),
            },
            setting: Setting {
                custom_setting: CustomSetting {
                    default_theme: model.setting_custom_setting_default_theme,
                    is_accepting_request: model.setting_custom_setting_is_accepting_request,
                },
                subscription: Subscription {
                    price_id: model.setting_subscription_price_id,
                    product_id: model.setting_subscription_product_id,
                    status: serde_json::from_str(&model.setting_subscription_status)
                        .unwrap_or(SubscriptionStatus::BASIC),
                    start_date: model.setting_subscription_start_date.map(DateTime::<Utc>::from),
                    end_date: model.setting_subscription_end_date.map(DateTime::<Utc>::from),
                },
            },
            timestamps,
        }
    }
}

impl From<User> for ActiveModel {
    fn from(user: User) -> Self {
        Self {
            id: Set(user.id),
            personal_first_name: Set(user.personal.first_name),
            personal_second_name: Set(user.personal.second_name),
            personal_email_address: Set(user.personal.email_address),
            personal_profile_image: Set(user.personal.profile_image),
            personal_username: Set(user.personal.username),
            password: Set(user.password),
            peripheral_authentication_code: Set(user.peripheral.authentication_code),
            peripheral_authentication_token: Set(user.peripheral.authentication_token),
            peripheral_timeout: Set(user.peripheral.timeout.map(|t| t.into())),
            peripheral_is_banned: Set(user.peripheral.is_banned),
            peripheral_is_verified: Set(user.peripheral.is_verified),
            verification_code: Set(user.verification.code),
            verification_timeout: Set(user.verification.timeout.map(|t| t.timestamp())),
            setting_custom_setting_default_theme: Set(user.setting.custom_setting.default_theme),
            setting_custom_setting_is_accepting_request: Set(user.setting.custom_setting.is_accepting_request),
            setting_subscription_price_id: Set(user.setting.subscription.price_id),
            setting_subscription_product_id: Set(user.setting.subscription.product_id),
            setting_subscription_status: Set(serde_json::to_string(&user.setting.subscription.status).unwrap()),
            setting_subscription_start_date: Set(user.setting.subscription.start_date.map(|t| t.into())),
            setting_subscription_end_date: Set(user.setting.subscription.end_date.map(|t| t.into())),
            created_at: Set(user.timestamps.created_at.into()),
            updated_at: Set(user.timestamps.updated_at.into()),
            deleted_at: Set(user.timestamps.deleted_at.map(|t| t.into())),
        }
    }
}

