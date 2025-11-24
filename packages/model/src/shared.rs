use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationOptions {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(10),
            sort_by: None,
            sort_order: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
    pub has_next: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, page: i32, limit: i32) -> Self {
        let has_next = i64::from(page) * i64::from(limit) < total;
        Self { items, total, page, limit, has_next }
    }
}

// Back-compat: re-export legacy path `crate::shared::data::models` to `crate::models`
pub mod data {
    pub use crate::models;
}