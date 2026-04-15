use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

/// Helper to build Redis session key
pub fn session_key(session_id: &Uuid) -> String {
    format!("session:{}", session_id)
}

/// Pagination query parameters from request
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

/// Pagination metadata for responses
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u64,
    pub limit: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl PaginationMeta {
    pub fn new(page: u64, limit: u64, total: u64) -> Self {
        let page = page.max(1);
        let limit = limit.clamp(1, 100);
        let total_pages = if total == 0 {
            1
        } else {
            (total + limit - 1) / limit
        };
        Self {
            page,
            limit,
            total,
            total_pages,
        }
    }
}

/// Internal pagination parameters
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: u64,
    pub limit: u64,
}

impl Pagination {
    pub fn new(page: u64, limit: u64) -> Self {
        Self {
            page: page.max(1),
            limit: limit.clamp(1, 100),
        }
    }

    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.limit
    }
}

impl From<PaginationQuery> for Pagination {
    fn from(query: PaginationQuery) -> Self {
        Self::new(query.page.unwrap_or(1), query.limit.unwrap_or(10))
    }
}

/// Helper to validate garde-annotated requests
pub fn validate_request<T: garde::Validate<Context = ()>>(request: &T) -> Result<(), AppError> {
    request.validate_with(&()).map_err(|e| {
        let mut fields = HashMap::new();
        for (path, error) in e.iter() {
            fields.insert(path.to_string(), error.to_string());
        }
        AppError::ValidationError { fields }
    })
}
