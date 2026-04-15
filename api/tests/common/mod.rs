use actix_web::{middleware, test, web, App, HttpMessage};
use api::config::global_state::AppState;
use api::config::settings::Config;
use api::middleware::request_context::RequestContext;
use api::routes;
use serde_json::Value;

/// Create a test app with full middleware stack.
/// Requires PostgreSQL and Redis running.
///
/// Loads configuration from `.env.test` only — no `.env` dependency.
/// Uses `from_filename_override` so `.env.test` values always take
/// precedence over any existing environment variables.
pub async fn create_test_app(
) -> impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>
{
    dotenvy::from_filename_override(".env.test").ok();
    let config = Config::from_env();
    let state = AppState::new(config)
        .await
        .expect("Failed to create test AppState");

    test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(middleware::from_fn(
                |req: actix_web::dev::ServiceRequest,
                 next: actix_web::middleware::Next<actix_web::body::BoxBody>| async move {
                    let ctx = RequestContext::new();
                    req.extensions_mut().insert(ctx);
                    next.call(req).await
                },
            ))
            .configure(routes::configure),
    )
    .await
}

/// Register a user and return (token, user_id)
pub async fn register_user(
    app: &impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>,
    name: &str,
    email: &str,
    password: &str,
) -> (String, String) {
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(serde_json::json!({
            "name": name,
            "email": email,
            "password": password
        }))
        .to_request();

    let resp = test::call_service(app, req).await;
    let body: Value = test::read_body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

/// Login and return token
pub async fn login_user(
    app: &impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>,
    email: &str,
    password: &str,
) -> String {
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(serde_json::json!({
            "email": email,
            "password": password
        }))
        .to_request();

    let resp = test::call_service(app, req).await;
    let body: Value = test::read_body_json(resp).await;
    body["token"].as_str().unwrap().to_string()
}

/// Create a project and return its ID
pub async fn create_project(
    app: &impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>,
    token: &str,
    name: &str,
) -> String {
    let req = test::TestRequest::post()
        .uri("/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "name": name,
            "description": "test project"
        }))
        .to_request();

    let resp = test::call_service(app, req).await;
    let body: Value = test::read_body_json(resp).await;
    body["id"].as_str().unwrap().to_string()
}

/// Create a task and return its ID
pub async fn create_task(
    app: &impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>,
    token: &str,
    project_id: &str,
    title: &str,
    priority: &str,
) -> String {
    let req = test::TestRequest::post()
        .uri(&format!("/projects/{}/tasks", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "title": title,
            "priority": priority
        }))
        .to_request();

    let resp = test::call_service(app, req).await;
    let body: Value = test::read_body_json(resp).await;
    body["id"].as_str().unwrap().to_string()
}

/// Generate a unique email for test isolation
pub fn unique_email(prefix: &str) -> String {
    format!("{}+{}@test.com", prefix, uuid::Uuid::new_v4())
}
