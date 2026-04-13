use actix_web::{web, HttpResponse};
use serde::Serialize;
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::{create_session, delete_session, generate_jwt, AuthenticatedUser};
use crate::storage::queries::user;
use crate::validation::custom::{validate_request, LoginRequest, RegisterRequest};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

/// POST /auth/register
pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    
    // Validate request
    validate_request(&body)?;

    // Check if email already exists
    if user::email_exists(&state.db, &body.email).await? {
        return Err(AppError::Conflict("email already exists".to_string()));
    }

    // Hash password
    let password_hash = bcrypt::hash(&body.password, state.config.bcrypt_cost)?;

    // Create user
    let user_id = Uuid::new_v4();
    let new_user = user::create(&state.db, user_id, &body.name, &body.email, &password_hash).await?;

    // Create session
    let session_id = Uuid::new_v4();
    create_session(&mut state.redis.clone(), &session_id, &user_id, state.config.jwt_expiry_hours).await?;

    // Generate JWT
    let token = generate_jwt(
        user_id,
        &body.email,
        session_id,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: UserResponse {
            id: new_user.id,
            name: new_user.name,
            email: new_user.email,
        },
    }))
}

/// POST /auth/login
pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    
    // Validate request
    validate_request(&body)?;

    // Find user by email
    let found_user = user::find_by_email(&state.db, &body.email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // Verify password
    let valid = bcrypt::verify(&body.password, &found_user.password)?;
    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    // Create session
    let session_id = Uuid::new_v4();
    create_session(
        &mut state.redis.clone(),
        &session_id,
        &found_user.id,
        state.config.jwt_expiry_hours,
    )
    .await?;

    // Generate JWT
    let token = generate_jwt(
        found_user.id,
        &found_user.email,
        session_id,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse {
            id: found_user.id,
            name: found_user.name,
            email: found_user.email,
        },
    }))
}

/// POST /auth/logout
pub async fn logout(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    // Delete session from Redis
    delete_session(&mut state.redis.clone(), &user.session_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "logged out successfully"
    })))
}
