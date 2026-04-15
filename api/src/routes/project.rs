use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::config::global_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::storage::queries::access;
use crate::storage::queries::project;
use crate::storage::queries::task;
use crate::types::common::{validate_request, Pagination, PaginationMeta, PaginationQuery};
use crate::types::project::{
    CreateProjectRequest, PaginatedProjectsResponse, ProjectDetailResponse, ProjectResponse,
    UpdateProjectRequest,
};
use crate::types::task::TaskResponse;

/// GET /projects
pub async fn list_projects(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let pagination: Pagination = query.into_inner().into();
    let result = project::list_for_user(&state.db, user.user_id, pagination).await?;

    Ok(HttpResponse::Ok().json(PaginatedProjectsResponse {
        projects: result.projects.into_iter().map(ProjectResponse::from).collect(),
        pagination: PaginationMeta::new(pagination.page, pagination.limit, result.total),
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
    let proj = access::check_project_access(&state.db, project_id, user.user_id).await?;

    let task_result = task::list(
        &state.db,
        project_id,
        task::TaskFilters {
            status: None,
            assignee_id: None,
        },
        Pagination::new(1, 1000),
    )
    .await?;

    Ok(HttpResponse::Ok().json(ProjectDetailResponse {
        project: ProjectResponse::from(proj),
        tasks: task_result.tasks.into_iter().map(TaskResponse::from).collect(),
    }))
}

/// PATCH /projects/{id}
pub async fn update_project(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProjectRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;

    let project_id = path.into_inner();
    let proj = access::check_owner(&state.db, project_id, user.user_id).await?;

    let updated = project::update(
        &state.db,
        proj,
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
    let proj = access::check_owner(&state.db, project_id, user.user_id).await?;

    project::soft_delete(&state.db, proj).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// GET /projects/{id}/stats
pub async fn get_project_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let project_id = path.into_inner();
    access::check_project_access(&state.db, project_id, user.user_id).await?;

    let stats = project::get_stats(&state.db, project_id).await?;

    Ok(HttpResponse::Ok().json(stats))
}
