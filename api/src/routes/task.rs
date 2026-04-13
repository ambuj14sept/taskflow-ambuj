use actix_web::{web, HttpResponse};
use serde::Serialize;
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::storage::queries::project;
use crate::storage::queries::task;
use crate::validation::custom::{CreateTaskRequest, TaskFilterQuery, UpdateTaskRequest};

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub project_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub due_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::storage::entities::task::Model> for TaskResponse {
    fn from(model: crate::storage::entities::task::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            description: model.description,
            status: model.status,
            priority: model.priority,
            project_id: model.project_id,
            assignee_id: model.assignee_id,
            creator_id: model.creator_id,
            due_date: model.due_date,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedTasksResponse {
    pub tasks: Vec<TaskResponse>,
    pub pagination: super::project::PaginationMeta,
}

/// GET /projects/{id}/tasks
pub async fn list_tasks(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    query: web::Query<TaskFilterQuery>,
) -> Result<HttpResponse, AppError> {
    // Validate filters
    query.validate()?;

    let project_id = path.into_inner();
    let filters = query.to_filters();
    let pagination = query.to_pagination();

    // Verify project exists and user has access
    let project = project::find_by_id(&state.db, project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check access
    if project.owner_id != user.user_id {
        // Check if user has tasks in this project
        let user_tasks = task::list(
            &state.db,
            project_id,
            task::TaskFilters {
                status: None,
                assignee_id: Some(user.user_id),
            },
            project::Pagination::new(1, 1),
        )
        .await?;

        if user_tasks.total == 0 {
            return Err(AppError::NotFound);
        }
    }

    let result = task::list(&state.db, project_id, filters, pagination).await?;

    let total_pages = (result.total + pagination.limit - 1) / pagination.limit;

    Ok(HttpResponse::Ok().json(PaginatedTasksResponse {
        tasks: result.tasks.into_iter().map(TaskResponse::from).collect(),
        pagination: super::project::PaginationMeta {
            page: pagination.page,
            limit: pagination.limit,
            total: result.total,
            total_pages,
        },
    }))
}

/// POST /projects/{id}/tasks
pub async fn create_task(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<CreateTaskRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    body.validate()?;

    let project_id = path.into_inner();

    // Verify project exists and user has access
    let project = project::find_by_id(&state.db, project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Only project owner or members with tasks can create tasks
    if project.owner_id != user.user_id {
        // Check if user has tasks in this project (is a member)
        let user_tasks = task::list(
            &state.db,
            project_id,
            task::TaskFilters {
                status: None,
                assignee_id: Some(user.user_id),
            },
            project::Pagination::new(1, 1),
        )
        .await?;

        if user_tasks.total == 0 {
            return Err(AppError::Forbidden);
        }
    }

    let task_id = Uuid::new_v4();
    let new_task = task::create(
        &state.db,
        task_id,
        &body.title,
        body.description.as_deref(),
        "todo", // Default status
        body.get_priority(),
        project_id,
        body.assignee_id,
        user.user_id, // creator_id
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
    // Validate request
    body.validate()?;

    let task_id = path.into_inner();
    let existing_task = task::find_by_id(&state.db, task_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Verify user has access to the project
    let project = project::find_by_id(&state.db, existing_task.project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check access: owner, assignee, or creator can update
    if project.owner_id != user.user_id
        && existing_task.assignee_id != Some(user.user_id)
        && existing_task.creator_id != user.user_id
    {
        return Err(AppError::Forbidden);
    }

    let updated = task::update(
        &state.db,
        existing_task,
        body.title.as_deref(),
        body.description.as_deref(),
        body.status.as_deref(),
        body.priority.as_deref(),
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

    // Check if user can delete (project owner or task creator)
    if !task::can_delete(&state.db, task_id, user.user_id).await? {
        // Task might not exist, or user doesn't have permission
        // Return 404 to avoid leaking existence
        return Err(AppError::NotFound);
    }

    task::delete(&state.db, task_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
