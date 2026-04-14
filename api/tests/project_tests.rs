mod common;

use actix_web::test;
use serde_json::Value;

use common::*;

#[actix_rt::test]
async fn test_create_project() {
    let app = create_test_app().await;
    let email = unique_email("proj_create");
    let (token, user_id) = register_user(&app, "Project Owner", &email, "password123").await;

    let req = test::TestRequest::post()
        .uri("/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "name": "Test Project",
            "description": "A test project"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "Test Project");
    assert_eq!(body["description"], "A test project");
    assert_eq!(body["owner_id"], user_id);
    assert!(body["id"].is_string());
}

#[actix_rt::test]
async fn test_create_project_validation() {
    let app = create_test_app().await;
    let email = unique_email("proj_val");
    let (token, _) = register_user(&app, "User", &email, "password123").await;

    // Empty name
    let req = test::TestRequest::post()
        .uri("/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "name": ""
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_list_projects() {
    let app = create_test_app().await;
    let email = unique_email("proj_list");
    let (token, _) = register_user(&app, "User", &email, "password123").await;

    // Create 2 projects
    create_project(&app, &token, "Project A").await;
    create_project(&app, &token, "Project B").await;

    let req = test::TestRequest::get()
        .uri("/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["projects"].as_array().unwrap().len() >= 2);
    assert!(body["pagination"]["total"].as_u64().unwrap() >= 2);
    assert_eq!(body["pagination"]["page"], 1);
}

#[actix_rt::test]
async fn test_list_projects_pagination() {
    let app = create_test_app().await;
    let email = unique_email("proj_page");
    let (token, _) = register_user(&app, "User", &email, "password123").await;

    // Create 3 projects
    create_project(&app, &token, "P1").await;
    create_project(&app, &token, "P2").await;
    create_project(&app, &token, "P3").await;

    // Request page 1 with limit 2
    let req = test::TestRequest::get()
        .uri("/projects?page=1&limit=2")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["projects"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["limit"], 2);
    assert!(body["pagination"]["total_pages"].as_u64().unwrap() >= 2);
}

#[actix_rt::test]
async fn test_get_project_detail() {
    let app = create_test_app().await;
    let email = unique_email("proj_detail");
    let (token, _) = register_user(&app, "User", &email, "password123").await;

    let project_id = create_project(&app, &token, "Detail Project").await;

    // Create a task in the project
    create_task(&app, &token, &project_id, "Task 1", "high").await;

    let req = test::TestRequest::get()
        .uri(&format!("/projects/{}", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "Detail Project");
    assert_eq!(body["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(body["tasks"][0]["title"], "Task 1");
}

#[actix_rt::test]
async fn test_get_project_not_found() {
    let app = create_test_app().await;
    let email = unique_email("proj_404");
    let (token, _) = register_user(&app, "User", &email, "password123").await;

    let req = test::TestRequest::get()
        .uri("/projects/00000000-0000-0000-0000-000000000000")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_rt::test]
async fn test_update_project_owner_only() {
    let app = create_test_app().await;
    let email1 = unique_email("proj_owner");
    let email2 = unique_email("proj_other");
    let (token1, _) = register_user(&app, "Owner", &email1, "password123").await;
    let (token2, _) = register_user(&app, "Other", &email2, "password123").await;

    let project_id = create_project(&app, &token1, "Owner's Project").await;

    // Owner can update
    let req = test::TestRequest::patch()
        .uri(&format!("/projects/{}", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token1)))
        .set_json(serde_json::json!({"name": "Updated Name"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "Updated Name");

    // Non-owner gets 403
    let req = test::TestRequest::patch()
        .uri(&format!("/projects/{}", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token2)))
        .set_json(serde_json::json!({"name": "Hacked"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

#[actix_rt::test]
async fn test_delete_project_owner_only() {
    let app = create_test_app().await;
    let email1 = unique_email("proj_del_own");
    let email2 = unique_email("proj_del_oth");
    let (token1, _) = register_user(&app, "Owner", &email1, "password123").await;
    let (token2, _) = register_user(&app, "Other", &email2, "password123").await;

    let project_id = create_project(&app, &token1, "To Delete").await;

    // Non-owner gets 403
    let req = test::TestRequest::delete()
        .uri(&format!("/projects/{}", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token2)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    // Owner can delete
    let req = test::TestRequest::delete()
        .uri(&format!("/projects/{}", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token1)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    // Verify it's gone
    let req = test::TestRequest::get()
        .uri(&format!("/projects/{}", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token1)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_rt::test]
async fn test_project_stats() {
    let app = create_test_app().await;
    let email = unique_email("proj_stats");
    let (token, _) = register_user(&app, "User", &email, "password123").await;

    let project_id = create_project(&app, &token, "Stats Project").await;

    // Create tasks with different statuses
    let task1 = create_task(&app, &token, &project_id, "Todo 1", "high").await;
    let task2 = create_task(&app, &token, &project_id, "Todo 2", "medium").await;
    let task3 = create_task(&app, &token, &project_id, "Todo 3", "low").await;

    // Update task2 to in_progress
    let req = test::TestRequest::patch()
        .uri(&format!("/tasks/{}", task2))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({"status": "in_progress"}))
        .to_request();
    test::call_service(&app, req).await;

    // Update task3 to done
    let req = test::TestRequest::patch()
        .uri(&format!("/tasks/{}", task3))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({"status": "done"}))
        .to_request();
    test::call_service(&app, req).await;

    // Get stats
    let req = test::TestRequest::get()
        .uri(&format!("/projects/{}/stats", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["total"], 3);
    assert_eq!(body["by_status"]["todo"], 1);
    assert_eq!(body["by_status"]["in_progress"], 1);
    assert_eq!(body["by_status"]["done"], 1);
}
