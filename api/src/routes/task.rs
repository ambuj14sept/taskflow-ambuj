use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::storage::queries::access;
use crate::storage::queries::task;
use crate::types::common::{validate_request, PaginationMeta};
use crate::types::enums::TaskStatus;
use crate::types::task::{
    CreateTaskRequest, PaginatedTasksResponse, TaskFilterQuery, TaskResponse, UpdateTaskRequest,
};

/// GET /projects/{id}/tasks
pub async fn list_tasks(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    query: web::Query<TaskFilterQuery>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let query = query.into_inner();
    let pagination = query.to_pagination();

    // Single access check — no redundant queries
    access::check_project_access(&state.db, project_id, user.user_id).await?;

    let filters = task::TaskFilters {
        status: query.status,
        assignee_id: query.assignee,
    };

    let result = task::list(&state.db, project_id, filters, pagination).await?;

    Ok(HttpResponse::Ok().json(PaginatedTasksResponse {
        tasks: result.tasks.into_iter().map(TaskResponse::from).collect(),
        pagination: PaginationMeta::new(pagination.page, pagination.limit, result.total),
    }))
}

/// POST /projects/{id}/tasks
pub async fn create_task(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<CreateTaskRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    validate_request(&body)?;

    let project_id = path.into_inner();

    // Verify project exists and user has access
    access::check_project_access(&state.db, project_id, user.user_id).await?;

    let task_id = Uuid::new_v4();
    let new_task = task::create(
        &state.db,
        task_id,
        &body.title,
        body.description.as_deref(),
        TaskStatus::default(),
        body.get_priority(),
        project_id,
        body.assignee_id,
        user.user_id,
        body.due_date,
    )
    .await?;

    Ok(HttpResponse::Created().json(TaskResponse::from(new_task)))
}

/// PATCH /tasks/{id}
pub async fn update_task(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTaskRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;

    let task_id = path.into_inner();
    let existing_task = task::find_by_id(&state.db, task_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check access: owner, assignee, or creator can update
    access::check_project_access(&state.db, existing_task.project_id, user.user_id).await?;

    let updated = task::update(
        &state.db,
        existing_task,
        body.title.as_deref(),
        body.description.as_deref(),
        body.status.as_ref(),
        body.priority.as_ref(),
        body.assignee_id,
        body.due_date,
    )
    .await?;

    Ok(HttpResponse::Ok().json(TaskResponse::from(updated)))
}

/// DELETE /tasks/{id}
pub async fn delete_task(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let task_id = path.into_inner();

    // Single query: find task + check permission (creator or project owner)
    let existing = task::check_delete_permission(&state.db, task_id, user.user_id).await?;

    task::soft_delete(&state.db, existing).await?;

    Ok(HttpResponse::NoContent().finish())
}
