pub mod auth;
pub mod project;
pub mod task;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(auth::register))
            .route("/login", web::post().to(auth::login))
            .route("/logout", web::post().to(auth::logout)),
    )
    .service(
        web::scope("/projects")
            .route("", web::get().to(project::list_projects))
            .route("", web::post().to(project::create_project))
            .route("/{id}", web::get().to(project::get_project))
            .route("/{id}", web::patch().to(project::update_project))
            .route("/{id}", web::delete().to(project::delete_project))
            .route("/{id}/stats", web::get().to(project::get_project_stats))
            .route("/{id}/tasks", web::get().to(task::list_tasks))
            .route("/{id}/tasks", web::post().to(task::create_task)),
    )
    .service(
        web::scope("/tasks")
            .route("/{id}", web::patch().to(task::update_task))
            .route("/{id}", web::delete().to(task::delete_task)),
    )
    .route("/health", web::get().to(health_check));
}

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}
