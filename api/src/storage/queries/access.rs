use sea_orm::*;
use uuid::Uuid;

use crate::errors::AppError;
use crate::storage::entities::{project, task};

/// Check if a user has access to a project (is owner or has tasks in it).
/// Returns the project if access is granted, NotFound if not.
pub async fn check_project_access(
    db: &DatabaseConnection,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<project::Model, AppError> {
    let proj = project::Entity::find_by_id(project_id)
        .filter(project::Column::IsActive.eq(true))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;

    if proj.owner_id == user_id {
        return Ok(proj);
    }

    // Check if user has any active tasks in this project
    let has_tasks = task::Entity::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .filter(task::Column::IsActive.eq(true))
        .filter(
            task::Column::AssigneeId
                .eq(user_id)
                .or(task::Column::CreatorId.eq(user_id)),
        )
        .count(db)
        .await?;

    if has_tasks > 0 {
        return Ok(proj);
    }

    Err(AppError::NotFound)
}

/// Check if a user is the owner of a project.
/// Returns the project if they are, appropriate error if not.
pub async fn check_owner(
    db: &DatabaseConnection,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<project::Model, AppError> {
    let proj = project::Entity::find_by_id(project_id)
        .filter(project::Column::IsActive.eq(true))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;

    if proj.owner_id != user_id {
        return Err(AppError::Forbidden);
    }

    Ok(proj)
}
