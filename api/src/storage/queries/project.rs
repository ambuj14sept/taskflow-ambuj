use sea_orm::*;
use uuid::Uuid;

use crate::storage::entities::{project, task};

#[derive(Clone, Copy)]
pub struct Pagination {
    pub page: u64,
    pub limit: u64,
}

impl Pagination {
    pub fn new(page: u64, limit: u64) -> Self {
        Self {
            page: page.max(1),
            limit: limit.min(100).max(1),
        }
    }

    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.limit
    }
}

pub struct ProjectListResult {
    pub projects: Vec<project::Model>,
    pub total: u64,
}

/// List projects the user owns or has tasks in
pub async fn list_for_user(
    db: &DatabaseConnection,
    user_id: Uuid,
    pagination: Pagination,
) -> Result<ProjectListResult, DbErr> {
    // Projects owned by user
    let owned_ids: Vec<Uuid> = project::Entity::find()
        .filter(project::Column::OwnerId.eq(user_id))
        .select_only()
        .column(project::Column::Id)
        .into_tuple()
        .all(db)
        .await?;

    // Projects where user has tasks (as assignee or creator)
    let has_tasks_ids: Vec<Uuid> = project::Entity::find()
        .inner_join(task::Entity)
        .filter(
            task::Column::AssigneeId.eq(user_id)
                .or(task::Column::CreatorId.eq(user_id)),
        )
        .select_only()
        .column(project::Column::Id)
        .into_tuple()
        .all(db)
        .await?;

    // Combine and deduplicate project IDs
    let mut project_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    project_ids.extend(owned_ids);
    project_ids.extend(has_tasks_ids);

    let total = project_ids.len() as u64;
    let project_ids: Vec<Uuid> = project_ids.into_iter().collect();

    let projects = project::Entity::find()
        .filter(project::Column::Id.is_in(project_ids))
        .order_by_desc(project::Column::CreatedAt)
        .offset(pagination.offset())
        .limit(pagination.limit)
        .all(db)
        .await?;

    Ok(ProjectListResult { projects, total })
}

pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<project::Model>, DbErr> {
    project::Entity::find_by_id(id).one(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    id: Uuid,
    name: &str,
    description: Option<&str>,
    owner_id: Uuid,
) -> Result<project::Model, DbErr> {
    let project = project::ActiveModel {
        id: Set(id),
        name: Set(name.to_string()),
        description: Set(description.map(|s| s.to_string())),
        owner_id: Set(owner_id),
        created_at: Set(chrono::Utc::now()),
    };

    project.insert(db).await
}

pub async fn update(
    db: &DatabaseConnection,
    project: project::Model,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<project::Model, DbErr> {
    let mut active: project::ActiveModel = project.into();

    if let Some(n) = name {
        active.name = Set(n.to_string());
    }
    if let Some(d) = description {
        active.description = Set(Some(d.to_string()));
    }

    active.update(db).await
}

pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<DeleteResult, DbErr> {
    project::Entity::delete_by_id(id).exec(db).await
}

pub async fn is_owner(db: &DatabaseConnection, project_id: Uuid, user_id: Uuid) -> Result<bool, DbErr> {
    let project = project::Entity::find_by_id(project_id)
        .one(db)
        .await?;

    Ok(project.map(|p| p.owner_id == user_id).unwrap_or(false))
}

/// Task counts by status and assignee for a project
#[derive(Debug, serde::Serialize)]
pub struct ProjectStats {
    pub by_status: std::collections::HashMap<String, u64>,
    pub by_assignee: std::collections::HashMap<String, u64>,
    pub total: u64,
}

pub async fn get_stats(db: &DatabaseConnection, project_id: Uuid) -> Result<ProjectStats, DbErr> {
    // Get all tasks for the project
    let tasks = task::Entity::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .all(db)
        .await?;

    let total = tasks.len() as u64;

    let mut by_status = std::collections::HashMap::new();
    let mut by_assignee = std::collections::HashMap::new();

    for task in &tasks {
        *by_status.entry(task.status.clone()).or_insert(0) += 1;
        
        let assignee_key = task.assignee_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unassigned".to_string());
        *by_assignee.entry(assignee_key).or_insert(0) += 1;
    }

    Ok(ProjectStats {
        by_status,
        by_assignee,
        total,
    })
}
