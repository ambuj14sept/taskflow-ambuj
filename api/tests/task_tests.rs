mod common;

use actix_web::test;
use serde_json::Value;

use common::*;

#[actix_rt::test]
async fn test_create_task() {
    let app = create_test_app().await;
    let email = unique_email("task_create");
    let (token, user_id) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Task Project").await;

    let req = test::TestRequest::post()
        .uri(&format!("/projects/{}/tasks", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "title": "My Task",
            "description": "Do something",
            "priority": "high",
            "due_date": "2026-05-01"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "My Task");
    assert_eq!(body["description"], "Do something");
    assert_eq!(body["status"], "todo"); // default
    assert_eq!(body["priority"], "high");
    assert_eq!(body["project_id"], project_id);
    assert_eq!(body["creator_id"], user_id);
    assert_eq!(body["due_date"], "2026-05-01");
    assert!(body["assignee_id"].is_null());
}

#[actix_rt::test]
async fn test_create_task_default_priority() {
    let app = create_test_app().await;
    let email = unique_email("task_defpri");
    let (token, _) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Project").await;

    let req = test::TestRequest::post()
        .uri(&format!("/projects/{}/tasks", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "title": "No priority set"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["priority"], "medium"); // default
}

#[actix_rt::test]
async fn test_create_task_validation() {
    let app = create_test_app().await;
    let email = unique_email("task_val");
    let (token, _) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Project").await;

    // Empty title
    let req = test::TestRequest::post()
        .uri(&format!("/projects/{}/tasks", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "title": "",
            "priority": "invalid"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "validation failed");
    assert!(body["fields"]["title"].is_string());
    assert!(body["fields"]["priority"].is_string());
}

#[actix_rt::test]
async fn test_list_tasks_with_filters() {
    let app = create_test_app().await;
    let email = unique_email("task_filter");
    let (token, _) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Filter Project").await;

    // Create tasks
    create_task(&app, &token, &project_id, "Task A", "high").await;
    let task_b = create_task(&app, &token, &project_id, "Task B", "low").await;

    // Move task_b to done
    let req = test::TestRequest::patch()
        .uri(&format!("/tasks/{}", task_b))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({"status": "done"}))
        .to_request();
    test::call_service(&app, req).await;

    // Filter by status=todo
    let req = test::TestRequest::get()
        .uri(&format!("/projects/{}/tasks?status=todo", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    let tasks = body["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Task A");
    assert_eq!(tasks[0]["status"], "todo");

    // Filter by status=done
    let req = test::TestRequest::get()
        .uri(&format!("/projects/{}/tasks?status=done", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    let tasks = body["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Task B");
}

#[actix_rt::test]
async fn test_list_tasks_pagination() {
    let app = create_test_app().await;
    let email = unique_email("task_page");
    let (token, _) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Page Project").await;

    // Create 3 tasks
    create_task(&app, &token, &project_id, "T1", "low").await;
    create_task(&app, &token, &project_id, "T2", "medium").await;
    create_task(&app, &token, &project_id, "T3", "high").await;

    // Page 1, limit 2
    let req = test::TestRequest::get()
        .uri(&format!("/projects/{}/tasks?page=1&limit=2", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["tasks"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"], 3);
    assert_eq!(body["pagination"]["total_pages"], 2);
}

#[actix_rt::test]
async fn test_update_task() {
    let app = create_test_app().await;
    let email = unique_email("task_update");
    let (token, _) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Update Project").await;
    let task_id = create_task(&app, &token, &project_id, "Original", "low").await;

    let req = test::TestRequest::patch()
        .uri(&format!("/tasks/{}", task_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "title": "Updated Title",
            "status": "in_progress",
            "priority": "high"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["status"], "in_progress");
    assert_eq!(body["priority"], "high");
    // updated_at should be different from created_at
    assert_ne!(body["created_at"], body["updated_at"]);
}

#[actix_rt::test]
async fn test_update_task_invalid_status() {
    let app = create_test_app().await;
    let email = unique_email("task_badstat");
    let (token, _) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Project").await;
    let task_id = create_task(&app, &token, &project_id, "Task", "low").await;

    let req = test::TestRequest::patch()
        .uri(&format!("/tasks/{}", task_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({"status": "invalid_status"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["fields"]["status"].is_string());
}

#[actix_rt::test]
async fn test_delete_task_by_creator() {
    let app = create_test_app().await;
    let email = unique_email("task_del_cr");
    let (token, _) = register_user(&app, "Creator", &email, "password123").await;
    let project_id = create_project(&app, &token, "Project").await;
    let task_id = create_task(&app, &token, &project_id, "To Delete", "low").await;

    let req = test::TestRequest::delete()
        .uri(&format!("/tasks/{}", task_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);
}

#[actix_rt::test]
async fn test_delete_task_by_non_owner_non_creator() {
    let app = create_test_app().await;
    let email1 = unique_email("task_del_own");
    let email2 = unique_email("task_del_rnd");
    let (token1, _) = register_user(&app, "Owner", &email1, "password123").await;
    let (token2, _) = register_user(&app, "Random", &email2, "password123").await;

    let project_id = create_project(&app, &token1, "Owner's Project").await;
    let task_id = create_task(&app, &token1, &project_id, "Task", "low").await;

    // Random user tries to delete — should fail
    let req = test::TestRequest::delete()
        .uri(&format!("/tasks/{}", task_id))
        .insert_header(("Authorization", format!("Bearer {}", token2)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404); // 404 to avoid leaking existence
}

#[actix_rt::test]
async fn test_delete_project_cascades_tasks() {
    let app = create_test_app().await;
    let email = unique_email("task_cascade");
    let (token, _) = register_user(&app, "User", &email, "password123").await;

    let project_id = create_project(&app, &token, "Cascade Project").await;
    let task_id = create_task(&app, &token, &project_id, "Task", "low").await;

    // Delete project
    let req = test::TestRequest::delete()
        .uri(&format!("/projects/{}", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    // Task should no longer be accessible
    let req = test::TestRequest::patch()
        .uri(&format!("/tasks/{}", task_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({"title": "ghost"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_rt::test]
async fn test_filter_invalid_status() {
    let app = create_test_app().await;
    let email = unique_email("task_badfilt");
    let (token, _) = register_user(&app, "User", &email, "password123").await;
    let project_id = create_project(&app, &token, "Project").await;

    let req = test::TestRequest::get()
        .uri(&format!("/projects/{}/tasks?status=invalid", project_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
