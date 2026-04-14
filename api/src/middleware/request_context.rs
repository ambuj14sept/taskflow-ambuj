use actix_web::{HttpMessage, HttpRequest};
use uuid::Uuid;

use crate::middleware::auth::AuthenticatedUser;

/// Request context stored in request extensions
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: Uuid,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4(),
        }
    }
}

/// Helper to get request ID from request
pub fn get_request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<RequestContext>()
        .map(|ctx| ctx.request_id.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Helper to get session ID from request (if authenticated)
pub fn get_session_id(req: &HttpRequest) -> Option<String> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .map(|user| user.session_id.to_string())
}
