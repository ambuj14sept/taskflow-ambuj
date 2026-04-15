use sea_orm::prelude::Expr;
use sea_orm::*;
use uuid::Uuid;

use crate::storage::entities::{project, task};
use crate::types::common::Pagination;
use crate::types::project::ProjectStats;

pub struct ProjectListResult {
    pub projects: Vec<project::Model>,
    pub total: u64,
}

/// List projects the user owns or has active tasks in (active projects only)
pub async fn list_for_user(
    db: &DatabaseConnection,
    user_id: Uuid,
    pagination: Pagination,
) -> Result<ProjectListResult, DbErr> {
    let owned_ids: Vec<Uuid> = project::Entity::find()
        .filter(project::Column::OwnerId.eq(user_id))
        .filter(project::Column::IsActive.eq(true))
        .select_only()
        .column(project::Column::Id)
        .into_tuple()
        .all(db)
        .await?;

    let has_tasks_ids: Vec<Uuid> = project::Entity::find()
        .inner_join(task::Entity)
        .filter(project::Column::IsActive.eq(true))
        .filter(task::Column::IsActive.eq(true))
        .filter(
            task::Column::AssigneeId
                .eq(user_id)
                .or(task::Column::CreatorId.eq(user_id)),
        )
        .select_only()
        .column(project::Column::Id)
        .into_tuple()
        .all(db)
        .await?;

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

pub async fn create(
    db: &DatabaseConnection,
    id: Uuid,
    name: &str,
    description: Option<&str>,
    owner_id: Uuid,
) -> Result<project::Model, DbErr> {
    let active = project::ActiveModel {
        id: Set(id),
        name: Set(name.to_string()),
        description: Set(description.map(|s| s.to_string())),
        owner_id: Set(owner_id),
        created_at: Set(chrono::Utc::now()),
        is_active: Set(true),
    };

    active.insert(db).await
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

/// Soft delete — set is_active = false on project and all its tasks
pub async fn soft_delete(db: &DatabaseConnection, project: project::Model) -> Result<(), DbErr> {
    let project_id = project.id;
    let mut active: project::ActiveModel = project.into();
    active.is_active = Set(false);
    active.update(db).await?;

    task::Entity::update_many()
        .col_expr(task::Column::IsActive, Expr::value(false))
        .filter(task::Column::ProjectId.eq(project_id))
        .exec(db)
        .await?;

    Ok(())
}

/// Task counts by status and assignee for a project (active tasks only)
pub async fn get_stats(db: &DatabaseConnection, project_id: Uuid) -> Result<ProjectStats, DbErr> {
    let tasks = task::Entity::find()
        .filter(task::Column::ProjectId.eq(project_id))
        .filter(task::Column::IsActive.eq(true))
        .all(db)
        .await?;

    let total = tasks.len() as u64;
    let mut by_status = std::collections::HashMap::new();
    let mut by_assignee = std::collections::HashMap::new();

    for t in &tasks {
        *by_status.entry(t.status.clone()).or_insert(0) += 1;

        let assignee_key = t
            .assignee_id
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
