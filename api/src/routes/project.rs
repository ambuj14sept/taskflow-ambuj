use actix_web::{web, HttpResponse};
use serde::Serialize;
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::storage::queries::project::{self, Pagination};
use crate::validation::custom::{validate_request, CreateProjectRequest, PaginationQuery, UpdateProjectRequest};

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::storage::entities::project::Model> for ProjectResponse {
    fn from(model: crate::storage::entities::project::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            owner_id: model.owner_id,
            created_at: model.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedProjectsResponse {
    pub projects: Vec<ProjectResponse>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u64,
    pub limit: u64,
    pub total: u64,
    pub total_pages: u64,
}

/// GET /projects
pub async fn list_projects(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let pagination: Pagination = query.into_inner().into();
    let result = project::list_for_user(&state.db, user.user_id, pagination).await?;

    let total_pages = (result.total + pagination.limit - 1) / pagination.limit;

    Ok(HttpResponse::Ok().json(PaginatedProjectsResponse {
        projects: result.projects.into_iter().map(ProjectResponse::from).collect(),
        pagination: PaginationMeta {
            page: pagination.page,
            limit: pagination.limit,
            total: result.total,
            total_pages,
        },
    }))
}

/// POST /projects
pub async fn create_project(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<CreateProjectRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    validate_request(&body)?;

    let project_id = Uuid::new_v4();
    let new_project = project::create(
        &state.db,
        project_id,
        &body.name,
        body.description.as_deref(),
        user.user_id,
    )
    .await?;

    Ok(HttpResponse::Created().json(ProjectResponse::from(new_project)))
}

/// GET /projects/{id}
pub async fn get_project(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    let project = project::find_by_id(&state.db, project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check if user has access (owner or has tasks in project)
    let tasks = crate::storage::queries::task::list(
        &state.db,
        project_id,
        crate::storage::queries::task::TaskFilters {
            status: None,
            assignee_id: None,
        },
        Pagination::new(1, 1000), // Get all tasks for the response
    )
    .await?;

    // Verify access
    let has_access = project.owner_id == user.user_id
        || tasks.tasks.iter().any(|t| {
            t.assignee_id == Some(user.user_id) || t.creator_id == user.user_id
        });

    if !has_access {
        return Err(AppError::NotFound);
    }

    #[derive(Debug, Serialize)]
    struct ProjectDetailResponse {
        #[serde(flatten)]
        project: ProjectResponse,
        tasks: Vec<TaskSummary>,
    }

    #[derive(Debug, Serialize)]
    struct TaskSummary {
        id: Uuid,
        title: String,
        description: Option<String>,
        status: String,
        priority: String,
        assignee_id: Option<Uuid>,
        creator_id: Uuid,
        due_date: Option<chrono::NaiveDate>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    Ok(HttpResponse::Ok().json(ProjectDetailResponse {
        project: ProjectResponse::from(project),
        tasks: tasks
            .tasks
            .into_iter()
            .map(|t| TaskSummary {
                id: t.id,
                title: t.title,
                description: t.description,
                status: t.status,
                priority: t.priority,
                assignee_id: t.assignee_id,
                creator_id: t.creator_id,
                due_date: t.due_date,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect(),
    }))
}

/// PATCH /projects/{id}
pub async fn update_project(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProjectRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    body.validate()?;

    let project_id = path.into_inner();
    let project = project::find_by_id(&state.db, project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check ownership
    if project.owner_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    let updated = project::update(
        &state.db,
        project,
        body.name.as_deref(),
        body.description.as_deref(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(ProjectResponse::from(updated)))
}

/// DELETE /projects/{id}
pub async fn delete_project(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();

    // Check project exists
    let proj = project::find_by_id(&state.db, project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check ownership
    if proj.owner_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    project::delete(&state.db, project_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// GET /projects/{id}/stats
pub async fn get_project_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();

    // Verify project exists and user has access
    let project = project::find_by_id(&state.db, project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check access
    if project.owner_id != user.user_id {
        // Check if user has tasks in this project
        let tasks = crate::storage::queries::task::list(
            &state.db,
            project_id,
            crate::storage::queries::task::TaskFilters {
                status: None,
                assignee_id: Some(user.user_id),
            },
            Pagination::new(1, 1),
        )
        .await?;

        if tasks.total == 0 {
            return Err(AppError::NotFound);
        }
    }

    let stats = project::get_stats(&state.db, project_id).await?;

    Ok(HttpResponse::Ok().json(stats))
}
