use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// POST /auth/register request body
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[garde(length(min = 1, max = 255))]
    pub name: String,

    #[garde(email)]
    pub email: String,

    #[garde(length(min = 8))]
    pub password: String,
}

/// POST /auth/login request body
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[garde(length(min = 1))]
    pub email: String,

    #[garde(length(min = 1))]
    pub password: String,
}

/// User info in auth responses (excludes password)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

/// Response for register and login
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}
