use actix_web::{dev::Payload, web, FromRequest, HttpMessage, HttpRequest};
use futures::future::{ready, Ready};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub session_id: Uuid,
}

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Get the authenticated user from request extensions (set by middleware)
        let user = req
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned();

        match user {
            Some(user) => ready(Ok(user)),
            None => ready(Err(AppError::Unauthorized)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: Uuid,
    pub email: String,
    pub session_id: Uuid,
    pub exp: usize,
}

/// Extract and validate JWT from Authorization header
pub fn extract_jwt(req: &HttpRequest, jwt_secret: &str) -> Result<JwtClaims, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized);
    }

    let token = &auth_header[7..];

    let token_data = jsonwebtoken::decode::<JwtClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Validate session exists in Redis
pub async fn validate_session(
    redis: &mut redis::aio::MultiplexedConnection,
    session_id: &Uuid,
) -> Result<Uuid, AppError> {
    let key = format!("session:{}", session_id);
    let user_id: Option<String> = redis.get(&key).await?;

    match user_id {
        Some(id) => {
            let user_id = Uuid::parse_str(&id)
                .map_err(|_| AppError::InternalError("Invalid session data".to_string()))?;
            Ok(user_id)
        }
        None => Err(AppError::Unauthorized),
    }
}

/// Auth middleware function
pub async fn auth_middleware(
    req: &HttpRequest,
    state: &web::Data<AppState>,
) -> Result<AuthenticatedUser, AppError> {
    let claims = extract_jwt(req, &state.config.jwt_secret)?;

    // Validate session in Redis
    let mut redis = state.redis.clone();
    validate_session(&mut redis, &claims.session_id).await?;

    Ok(AuthenticatedUser {
        user_id: claims.user_id,
        email: claims.email,
        session_id: claims.session_id,
    })
}

/// Generate a JWT token
pub fn generate_jwt(
    user_id: Uuid,
    email: &str,
    session_id: Uuid,
    jwt_secret: &str,
    expiry_hours: u64,
) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(expiry_hours as i64))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = JwtClaims {
        user_id,
        email: email.to_string(),
        session_id,
        exp: expiration,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

/// Create a session in Redis
pub async fn create_session(
    redis: &mut redis::aio::MultiplexedConnection,
    session_id: &Uuid,
    user_id: &Uuid,
    expiry_hours: u64,
) -> Result<(), AppError> {
    let key = format!("session:{}", session_id);
    let expiry_seconds = expiry_hours * 3600;

    let _: () = redis
        .set_ex(&key, user_id.to_string(), expiry_seconds)
        .await?;

    Ok(())
}

/// Delete a session from Redis (logout)
pub async fn delete_session(
    redis: &mut redis::aio::MultiplexedConnection,
    session_id: &Uuid,
) -> Result<(), AppError> {
    let key = format!("session:{}", session_id);
    let _: () = redis.del(&key).await?;
    Ok(())
}
