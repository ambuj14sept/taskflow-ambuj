use sea_orm::*;
use uuid::Uuid;

use crate::storage::entities::task;
use crate::storage::queries::project::Pagination;

pub struct TaskListResult {
    pub tasks: Vec<task::Model>,
    pub total: u64,
}

pub struct TaskFilters {
    pub status: Option<String>,
    pub assignee_id: Option<Uuid>,
}

pub async fn list(
    db: &DatabaseConnection,
    project_id: Uuid,
    filters: TaskFilters,
    pagination: Pagination,
) -> Result<TaskListResult, DbErr> {
    let mut query = task::Entity::find()
        .filter(task::Column::ProjectId.eq(project_id));

    if let Some(status) = filters.status {
        query = query.filter(task::Column::Status.eq(status));
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

pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<task::Model>, DbErr> {
    task::Entity::find_by_id(id).one(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    id: Uuid,
    title: &str,
    description: Option<&str>,
    status: &str,
    priority: String,
    project_id: Uuid,
    assignee_id: Option<Uuid>,
    creator_id: Uuid,
    due_date: Option<chrono::NaiveDate>,
) -> Result<task::Model, DbErr> {
    let now = chrono::Utc::now();

    let task = task::ActiveModel {
        id: Set(id),
        title: Set(title.to_string()),
        description: Set(description.map(|s| s.to_string())),
        status: Set(status.to_string()),
        priority: Set(priority),
        project_id: Set(project_id),
        assignee_id: Set(assignee_id),
        creator_id: Set(creator_id),
        due_date: Set(due_date),
        created_at: Set(now),
        updated_at: Set(now),
    };

    task.insert(db).await
}

pub async fn update(
    db: &DatabaseConnection,
    task: task::Model,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<&str>,
    priority: Option<&str>,
    assignee_id: Option<Option<Uuid>>,
    due_date: Option<Option<chrono::NaiveDate>>,
) -> Result<task::Model, DbErr> {
    let mut active: task::ActiveModel = task.into();

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

pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<DeleteResult, DbErr> {
    task::Entity::delete_by_id(id).exec(db).await
}

/// Check if user can delete a task (project owner or task creator)
pub async fn can_delete(
    db: &DatabaseConnection,
    task_id: Uuid,
    user_id: Uuid,
) -> Result<bool, DbErr> {
    let task = task::Entity::find_by_id(task_id)
        .find_also_related(crate::storage::entities::project::Entity)
        .one(db)
        .await?;

    match task {
        Some((task, Some(project))) => {
            // User is task creator or project owner
            Ok(task.creator_id == user_id || project.owner_id == user_id)
        }
        _ => Ok(false),
    }
}

/// Get project ID for a task
pub async fn get_project_id(db: &DatabaseConnection, task_id: Uuid) -> Result<Option<Uuid>, DbErr> {
    let task = task::Entity::find_by_id(task_id).one(db).await?;
    Ok(task.map(|t| t.project_id))
}
