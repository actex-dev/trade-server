use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug)]
pub enum EncryptionError {
    // #[error("hash error: {0}")]
    HashError(String),

    // #[error("verify error: {0}")]
    VerifyError(String),

    // #[error("jwt error: {0}")]
    JwtError(String),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    /// token expiration in seconds
    pub expiry_seconds: i64,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ArgonConfig {
    /// Number of iterations (time cost)
    pub t_cost: u32,
    pub m_cost_kib: u32,
    pub p_cost: u32,
}

/// Sub payload can be raw JSON or a JSON string (from other services)
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum Sub {
    Json(serde_json::Value),
    Text(String),
}

/// Claims example — extend as you need
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Claims {
    pub sub: Sub,
    pub exp: i64,
}

impl Claims {
    pub fn new<T: Serialize>(payload: &T, expiry_seconds: i64) -> Result<Self, serde_json::Error> {
        let sub = Sub::Json(serde_json::to_value(payload)?);
        let exp = chrono::Utc::now().timestamp() + expiry_seconds;
        Ok(Claims { sub, exp })
    }

    pub fn new_text<T: Serialize>(payload: &T, expiry_seconds: i64) -> Result<Self, serde_json::Error> {
        let sub = Sub::Text(serde_json::to_string(payload)?);
        let exp = chrono::Utc::now().timestamp() + expiry_seconds;
        Ok(Claims { sub, exp })
    }
}

#[derive(Clone, Debug)]
pub struct TokenParams {
    pub key: String,
    pub expiry_seconds: i64,
}

#[allow(dead_code)]
pub struct Token;

impl Token {
    pub fn user_access_token() -> TokenParams {
        TokenParams {
            key: std::env::var("USER_ACCESS_TOKEN").unwrap_or_else(|_| "default_user_access_token".to_string()),
            expiry_seconds: 72 * 3600, // 72 hours
        }
    }

    pub fn user_refresh_token() -> TokenParams {
        TokenParams {
            key: std::env::var("USER_REFRESH_TOKEN").unwrap_or_else(|_| "default_user_refresh_token".to_string()),
            expiry_seconds: 100 * 24 * 3600, // 100 days
        }
    }

    pub fn admin_access_token() -> TokenParams {
        TokenParams {
            key: std::env::var("ADMIN_SECRET_TOKEN").unwrap_or_else(|_| "default_admin_token".to_string()),
            expiry_seconds: 72 * 3600, // 72 hours
        }
    }

    pub fn web_access_token() -> TokenParams {
        TokenParams {
            key: std::env::var("WEB_ACCESS_TOKEN").unwrap_or_else(|_| "default_web_token".to_string()),
            expiry_seconds: 5 * 60, // 5 minutes
        }
    }

    pub fn app_access_token() -> TokenParams {
        TokenParams {
            key: std::env::var("APP_ACCESS_TOKEN").unwrap_or_else(|_| "default_app_token".to_string()),
            expiry_seconds: 6 * 3600, // 6 hours
        }
    }

    pub fn app_refresh_token() -> TokenParams {
        TokenParams {
            key: std::env::var("APP_REFRESH_TOKEN").unwrap_or_else(|_| "default_app_refresh_token".to_string()),
            expiry_seconds: 72 * 3600, // 72 hours
        }
    }
}