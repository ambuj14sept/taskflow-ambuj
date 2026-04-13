use actix_web::{dev::Service, dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage, web};
use futures::future::LocalBoxFuture;
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::middleware::auth::{auth_middleware, AuthenticatedUser};

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

/// Middleware that adds request context and handles authentication
pub fn request_context_middleware<S>(
    state: web::Data<AppState>,
) -> impl Fn(
    ServiceRequest,
    S,
) -> LocalBoxFuture<'static, Result<ServiceResponse, Error>>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S: 'static,
{
    move |req: ServiceRequest, service: S| {
        let state = state.clone();
        
        Box::pin(async move {
            // Generate request ID and store in extensions
            let ctx = RequestContext::new();
            let _request_id = ctx.request_id;
            req.extensions_mut().insert(ctx);

            // Check if this is a public route (no auth required)
            let path = req.path();
            let is_public_route = path == "/auth/register" || path == "/auth/login" || path == "/health";

            if !is_public_route {
                // Perform authentication
                // We need to pass the HttpRequest, not ServiceRequest
                let http_req = req.request();
                match auth_middleware(http_req, &state).await {
                    Ok(user) => {
                        req.extensions_mut().insert(user);
                    }
                    Err(e) => {
                        // For protected routes, return the error
                        return Err(e.into());
                    }
                }
            }

            // Call the next service
            service.call(req).await
        })
    }
}

/// Helper to get request ID from request
pub fn get_request_id(req: &actix_web::HttpRequest) -> String {
    req.extensions()
        .get::<RequestContext>()
        .map(|ctx| ctx.request_id.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Helper to get session ID from request (if authenticated)
pub fn get_session_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .map(|user| user.session_id.to_string())
}
