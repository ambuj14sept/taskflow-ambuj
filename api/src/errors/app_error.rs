use std::collections::HashMap;

use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed")]
    ValidationError { fields: HashMap<String, String> },

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
            _ => AppError::InternalError(err.to_string()),
        }
    }
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
