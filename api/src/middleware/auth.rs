use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;
use crate::types::common::session_key;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub session_id: Uuid,
}

/// Async FromRequest — extracts JWT from header, validates session in Redis.
/// No separate middleware needed. Actix calls this per-handler automatically.
impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let state = req
                .app_data::<web::Data<AppState>>()
                .ok_or(AppError::InternalError("AppState not configured".to_string()))?;

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
                &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
                &jsonwebtoken::Validation::default(),
            )
            .map_err(|_| AppError::Unauthorized)?;

            let claims = token_data.claims;

            let key = session_key(&claims.session_id);
            let mut redis = state.redis.clone();
            let stored_user_id: Option<String> = redis.get(&key).await?;

            match stored_user_id {
                Some(id) => {
                    let parsed_id = Uuid::parse_str(&id)
                        .map_err(|_| AppError::InternalError("Invalid session data".to_string()))?;

                    if parsed_id != claims.user_id {
                        return Err(AppError::Unauthorized);
                    }

                    Ok(AuthenticatedUser {
                        user_id: claims.user_id,
                        email: claims.email,
                        session_id: claims.session_id,
                    })
                }
                None => Err(AppError::Unauthorized),
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: Uuid,
    pub email: String,
    pub session_id: Uuid,
    pub exp: usize,
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
    let key = session_key(session_id);
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
    let key = session_key(session_id);
    let _: () = redis.del(&key).await?;
    Ok(())
}
