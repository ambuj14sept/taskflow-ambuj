use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::{create_session, delete_session, generate_jwt, AuthenticatedUser};
use crate::storage::queries::user;
use crate::types::auth::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::types::common::validate_request;

/// POST /auth/register
pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    validate_request(&body)?;

    if user::email_exists(&state.db, &body.email).await? {
        return Err(AppError::Conflict("email already exists".to_string()));
    }

    let password_hash = bcrypt::hash(&body.password, state.config.bcrypt_cost)?;

    let user_id = Uuid::new_v4();
    let new_user = user::create(&state.db, user_id, &body.name, &body.email, &password_hash).await?;

    let session_id = Uuid::new_v4();
    create_session(&mut state.redis.clone(), &session_id, &user_id, state.config.jwt_expiry_hours).await?;

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
    validate_request(&body)?;

    let found_user = user::find_by_email(&state.db, &body.email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let valid = bcrypt::verify(&body.password, &found_user.password)?;
    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let session_id = Uuid::new_v4();
    create_session(
        &mut state.redis.clone(),
        &session_id,
        &found_user.id,
        state.config.jwt_expiry_hours,
    )
    .await?;

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
    delete_session(&mut state.redis.clone(), &user.session_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "logged out successfully"
    })))
}
