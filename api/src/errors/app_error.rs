use std::collections::HashMap;

use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed")]
    ValidationError { fields: HashMap<String, String> },

    #[error("{0}")]
    BadRequest(String),

    #[error("invalid email or password")]
    InvalidCredentials,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal server error")]
    InternalError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::ValidationError { fields } => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "validation failed",
                    "fields": fields
                }))
            }
            AppError::BadRequest(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": msg
            })),
            AppError::InvalidCredentials => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid email or password"
            })),
            AppError::Unauthorized => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized"
            })),
            AppError::Forbidden => HttpResponse::Forbidden().json(serde_json::json!({
                "error": "forbidden"
            })),
            AppError::NotFound => HttpResponse::NotFound().json(serde_json::json!({
                "error": "not found"
            })),
            AppError::Conflict(msg) => HttpResponse::Conflict().json(serde_json::json!({
                "error": format!("conflict: {}", msg)
            })),
            AppError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "internal server error",
                    "details": msg
                }))
            }
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::ValidationError { .. } => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::InvalidCredentials => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::Forbidden => actix_web::http::StatusCode::FORBIDDEN,
            AppError::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            AppError::Conflict(_) => actix_web::http::StatusCode::CONFLICT,
            AppError::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        match err {
            sea_orm::DbErr::RecordNotFound(_) => AppError::NotFound,
            sea_orm::DbErr::Query(ref runtime_err) => {
                let msg = runtime_err.to_string();

                // PostgreSQL foreign key violation (error code 23503)
                if msg.contains("23503") || msg.contains("violates foreign key constraint") {
                    let field = extract_fk_field(&msg);
                    return AppError::BadRequest(format!("{} does not exist", field));
                }

                // PostgreSQL unique violation (error code 23505)
                if msg.contains("23505") || msg.contains("violates unique constraint") {
                    let field = extract_unique_field(&msg);
                    return AppError::Conflict(format!("{} already exists", field));
                }

                AppError::InternalError(msg)
            }
            _ => AppError::InternalError(err.to_string()),
        }
    }
}

/// Extract the field name from a PostgreSQL foreign key violation message.
///
/// Example input:  `insert or update on table "tasks" violates foreign key constraint "fk_tasks_assignee_id"`
/// Example output: `assignee_id`
fn extract_fk_field(msg: &str) -> String {
    // Constraint name format: fk_{table}_{column}
    if let Some(constraint) = msg.split('"').nth(1) {
        if let Some(column) = constraint.strip_prefix("fk_") {
            // Remove the table prefix: "fk_tasks_assignee_id" → "assignee_id"
            if let Some(underscore_pos) = column.find('_') {
                let rest = &column[underscore_pos + 1..];
                if !rest.is_empty() {
                    return rest.to_string();
                }
            }
            return column.to_string();
        }
    }
    "referenced record".to_string()
}

/// Extract the field name from a PostgreSQL unique violation message.
///
/// Example input:  `duplicate key value violates unique constraint "users_email_key"`
/// Example output: `email`
fn extract_unique_field(msg: &str) -> String {
    if let Some(constraint) = msg.split('"').nth(1) {
        // Constraint name format: {table}_{column}_key
        if let Some(without_suffix) = constraint.strip_suffix("_key") {
            // Remove the table prefix: "users_email_key" → "email"
            if let Some(underscore_pos) = without_suffix.find('_') {
                let rest = &without_suffix[underscore_pos + 1..];
                if !rest.is_empty() {
                    return rest.to_string();
                }
            }
            return without_suffix.to_string();
        }
    }
    "value".to_string()
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::InternalError(format!("Redis error: {}", err))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::InternalError(format!("Bcrypt error: {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized
    }
}
