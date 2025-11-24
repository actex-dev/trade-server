use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::shared::PaginatedResponse;

use crate::models::{Model, SoftDelete, Timestamps};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubscriptionStatus {
    PRO,
    BASIC,
    ENTERPRISE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub code: String,
    pub timeout: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub price_id: Option<String>,
    pub product_id: Option<String>,
    pub status: SubscriptionStatus,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personal {
    pub first_name: String,
    pub second_name: String,
    #[serde(rename = "email_address")]
    pub email_address: String,
    pub profile_image: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peripheral {
    #[serde(skip_serializing)]
    pub authentication_code: Option<String>,
    #[serde(skip_serializing)]
    pub authentication_token: Option<String>,
    pub timeout: Option<DateTime<Utc>>,
    pub is_banned: bool,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSetting {
    pub default_theme: Option<String>,
    pub is_accepting_request: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub custom_setting: CustomSetting,
    pub subscription: Subscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub personal: Personal,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub peripheral: Peripheral,
    #[serde(skip_serializing)]
    pub verification: Verification,
    pub setting: Setting,
    pub timestamps: Timestamps,
}

impl Model for User {
    fn id(&self) -> Uuid {
        self.id
    }

    fn timestamps(&self) -> &Timestamps {
        &self.timestamps
    }

    fn timestamps_mut(&mut self) -> &mut Timestamps {
        &mut self.timestamps
    }
}

impl SoftDelete for User {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleUser {
    pub total_users: i64,
    pub users: Vec<User>,
    pub has_next: bool,
}

// Request DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub first_name: String,
    pub second_name: String,
    pub email_address: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email_address: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResetCodeRequest {
    pub email_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResetCodeRequest {
    pub email_address: String,
    pub auth_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalRequest {
    pub email_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePersonal {
    pub first_name: String,
    pub second_name: String,
    pub email_address: String,
    pub profile_image: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCodeInjection {
    pub authentication_code: String,
    pub timeout: DateTime<Utc>,
    pub is_banned: bool,
}

// Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserResponse {
    pub id: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCodeResponse {
    pub token: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordAuthResponse {
    pub email_address: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureUserResponse {
    pub id: String,
    pub personal: Personal,
    pub timestamps: Timestamps,
    pub verification: Verification,
    pub setting: Setting,
}

impl From<User> for SecureUserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            personal: user.personal,
            timestamps: user.timestamps,
            verification: user.verification,
            setting: user.setting
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralUserResponse {
    pub id: String,
    pub personal: Personal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for GeneralUserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            personal: user.personal,
            created_at: user.timestamps.created_at,
            updated_at: user.timestamps.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleSecureResponse {
    pub total_users: i64,
    pub users: Vec<SecureUserResponse>,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleGeneralResponse {
    pub total_users: i64,
    pub users: Vec<GeneralUserResponse>,
    pub has_next: bool,
}

// Unified paginated response aliases
pub type SecureUsersPage = PaginatedResponse<SecureUserResponse>;
pub type GeneralUsersPage = PaginatedResponse<GeneralUserResponse>;
