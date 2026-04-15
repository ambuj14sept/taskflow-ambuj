use std::collections::HashMap;

use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::storage::entities::task;
use crate::types::common::{Pagination, PaginationMeta};
use crate::types::enums::{TaskPriority, TaskStatus};

/// POST /projects/:id/tasks request body
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTaskRequest {
    #[garde(length(min = 1, max = 255))]
    pub title: String,

    #[garde(skip)]
    pub description: Option<String>,

    #[garde(skip)]
    pub priority: Option<TaskPriority>,

    #[garde(skip)]
    pub assignee_id: Option<Uuid>,

    #[garde(skip)]
    pub due_date: Option<chrono::NaiveDate>,
}

impl CreateTaskRequest {
    pub fn get_priority(&self) -> TaskPriority {
        self.priority.unwrap_or_default()
    }
}

/// PATCH /tasks/:id request body
#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee_id: Option<Option<Uuid>>,
    pub due_date: Option<Option<chrono::NaiveDate>>,
}

impl UpdateTaskRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        let mut fields = HashMap::new();

        if self.title.is_none()
            && self.description.is_none()
            && self.status.is_none()
            && self.priority.is_none()
            && self.assignee_id.is_none()
            && self.due_date.is_none()
        {
            fields.insert(
                "request".to_string(),
                "at least one field is required".to_string(),
            );
        }

        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                fields.insert("title".to_string(), "cannot be empty".to_string());
            }
            if title.len() > 255 {
                fields.insert(
                    "title".to_string(),
                    "must be at most 255 characters".to_string(),
                );
            }
        }

        if !fields.is_empty() {
            return Err(AppError::ValidationError { fields });
        }

        Ok(())
    }
}

/// Task filter query parameters
#[derive(Debug, Deserialize)]
pub struct TaskFilterQuery {
    pub status: Option<TaskStatus>,
    pub assignee: Option<Uuid>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl TaskFilterQuery {
    pub fn to_pagination(&self) -> Pagination {
        Pagination::new(self.page.unwrap_or(1), self.limit.unwrap_or(10))
    }
}

/// Single task in responses
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

impl From<task::Model> for TaskResponse {
    fn from(model: task::Model) -> Self {
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

/// GET /projects/:id/tasks response with pagination
#[derive(Debug, Serialize)]
pub struct PaginatedTasksResponse {
    pub tasks: Vec<TaskResponse>,
    pub pagination: PaginationMeta,
}
