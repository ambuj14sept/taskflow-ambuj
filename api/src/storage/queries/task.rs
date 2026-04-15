use sea_orm::*;
use uuid::Uuid;

use crate::errors::AppError;
use crate::storage::entities::{project, task};
use crate::types::common::Pagination;
use crate::types::enums::{TaskPriority, TaskStatus};

pub struct TaskListResult {
    pub tasks: Vec<task::Model>,
    pub total: u64,
}

pub struct TaskFilters {
    pub status: Option<TaskStatus>,
    pub assignee_id: Option<Uuid>,
}

/// List active tasks for a project with optional filters
pub async fn list(
    db: &DatabaseConnection,
    project_id: Uuid,
    filters: TaskFilters,
    pagination: Pagination,
) -> Result<TaskListResult, DbErr> {
    let mut query = task::Entity::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .filter(task::Column::IsActive.eq(true));

    if let Some(status) = filters.status {
        query = query.filter(task::Column::Status.eq(status.to_string()));
    }

    if let Some(assignee_id) = filters.assignee_id {
        query = query.filter(task::Column::AssigneeId.eq(assignee_id));
    }

    let total = query.clone().count(db).await?;

    let tasks = query
        .order_by_desc(task::Column::CreatedAt)
        .offset(pagination.offset())
        .limit(pagination.limit)
        .all(db)
        .await?;

    Ok(TaskListResult { tasks, total })
}

/// Find an active task by ID
pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<task::Model>, DbErr> {
    task::Entity::find_by_id(id)
        .filter(task::Column::IsActive.eq(true))
        .one(db)
        .await
}

pub async fn create(
    db: &DatabaseConnection,
    id: Uuid,
    title: &str,
    description: Option<&str>,
    status: TaskStatus,
    priority: TaskPriority,
    project_id: Uuid,
    assignee_id: Option<Uuid>,
    creator_id: Uuid,
    due_date: Option<chrono::NaiveDate>,
) -> Result<task::Model, DbErr> {
    let now = chrono::Utc::now();

    let active = task::ActiveModel {
        id: Set(id),
        title: Set(title.to_string()),
        description: Set(description.map(|s| s.to_string())),
        status: Set(status.to_string()),
        priority: Set(priority.to_string()),
        project_id: Set(project_id),
        assignee_id: Set(assignee_id),
        creator_id: Set(creator_id),
        due_date: Set(due_date),
        created_at: Set(now),
        updated_at: Set(now),
        is_active: Set(true),
    };

    active.insert(db).await
}

pub async fn update(
    db: &DatabaseConnection,
    existing: task::Model,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<&TaskStatus>,
    priority: Option<&TaskPriority>,
    assignee_id: Option<Option<Uuid>>,
    due_date: Option<Option<chrono::NaiveDate>>,
) -> Result<task::Model, DbErr> {
    let mut active: task::ActiveModel = existing.into();

    if let Some(t) = title {
        active.title = Set(t.to_string());
    }
    if let Some(d) = description {
        active.description = Set(Some(d.to_string()));
    }
    if let Some(s) = status {
        active.status = Set(s.to_string());
    }
    if let Some(p) = priority {
        active.priority = Set(p.to_string());
    }
    if let Some(a) = assignee_id {
        active.assignee_id = Set(a);
    }
    if let Some(d) = due_date {
        active.due_date = Set(d);
    }

    active.updated_at = Set(chrono::Utc::now());

    active.update(db).await
}

/// Soft delete — set is_active = false
pub async fn soft_delete(db: &DatabaseConnection, existing: task::Model) -> Result<(), DbErr> {
    let mut active: task::ActiveModel = existing.into();
    active.is_active = Set(false);
    active.update(db).await?;
    Ok(())
}

/// Check if user can delete a task (project owner or task creator).
/// Returns the task if permitted, AppError if not.
pub async fn check_delete_permission(
    db: &DatabaseConnection,
    task_id: Uuid,
    user_id: Uuid,
) -> Result<task::Model, AppError> {
    let result = task::Entity::find_by_id(task_id)
        .filter(task::Column::IsActive.eq(true))
        .find_also_related(project::Entity)
        .one(db)
        .await?;

    match result {
        Some((t, Some(p))) if t.creator_id == user_id || p.owner_id == user_id => Ok(t),
        Some(_) => Err(AppError::NotFound), // Don't leak existence
        None => Err(AppError::NotFound),
    }
}
