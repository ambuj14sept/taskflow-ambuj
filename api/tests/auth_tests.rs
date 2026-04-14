mod common;

use actix_web::test;
use serde_json::Value;

use common::*;

#[actix_rt::test]
async fn test_register_success() {
    let app = create_test_app().await;
    let email = unique_email("register");

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(serde_json::json!({
            "name": "Test User",
            "email": email,
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["name"], "Test User");
    assert_eq!(body["user"]["email"], email);
    assert!(body["user"]["id"].is_string());
}

#[actix_rt::test]
async fn test_register_validation_errors() {
    let app = create_test_app().await;

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(serde_json::json!({
            "name": "",
            "email": "not-an-email",
            "password": "short"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "validation failed");
    assert!(body["fields"]["name"].is_string());
    assert!(body["fields"]["email"].is_string());
    assert!(body["fields"]["password"].is_string());
}

#[actix_rt::test]
async fn test_register_duplicate_email() {
    let app = create_test_app().await;
    let email = unique_email("dup");

    // First registration
    register_user(&app, "User1", &email, "password123").await;

    // Second registration with same email
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(serde_json::json!({
            "name": "User2",
            "email": email,
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("email already exists"));
}

#[actix_rt::test]
async fn test_login_success() {
    let app = create_test_app().await;
    let email = unique_email("login");

    register_user(&app, "Login User", &email, "password123").await;

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(serde_json::json!({
            "email": email,
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], email);
}

#[actix_rt::test]
async fn test_login_wrong_password() {
    let app = create_test_app().await;
    let email = unique_email("wrongpw");

    register_user(&app, "User", &email, "password123").await;

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(serde_json::json!({
            "email": email,
            "password": "wrongpassword"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "invalid email or password");
}

#[actix_rt::test]
async fn test_login_nonexistent_email() {
    let app = create_test_app().await;

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(serde_json::json!({
            "email": "nonexistent@test.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_logout_invalidates_session() {
    let app = create_test_app().await;
    let email = unique_email("logout");
    let (token, _) = register_user(&app, "Logout User", &email, "password123").await;

    // Logout
    let req = test::TestRequest::post()
        .uri("/auth/logout")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Try to access protected route with the same token
    let req = test::TestRequest::get()
        .uri("/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_protected_route_no_token() {
    let app = create_test_app().await;

    let req = test::TestRequest::get()
        .uri("/projects")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_protected_route_invalid_token() {
    let app = create_test_app().await;

    let req = test::TestRequest::get()
        .uri("/projects")
        .insert_header(("Authorization", "Bearer invalid.jwt.token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}
