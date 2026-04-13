use std::collections::HashMap;

use garde::Validate;
use serde::Deserialize;

use crate::errors::AppError;

// ============ Auth Request DTOs ============

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[garde(length(min = 1, max = 255))]
    pub name: String,

    #[garde(email)]
    pub email: String,

    #[garde(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[garde(length(min = 1))]
    pub email: String,

    #[garde(length(min = 1))]
    pub password: String,
}

// ============ Project Request DTOs ============

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[garde(length(min = 1, max = 255))]
    pub name: String,

    #[garde(skip)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl UpdateProjectRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        let mut fields = HashMap::new();

        if self.name.is_none() && self.description.is_none() {
            fields.insert(
                "request".to_string(),
                "at least one field is required".to_string(),
            );
        }

        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                fields.insert("name".to_string(), "cannot be empty".to_string());
            }
            if name.len() > 255 {
                fields.insert(
                    "name".to_string(),
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

// ============ Task Request DTOs ============

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<uuid::Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
}

impl CreateTaskRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        let mut fields = HashMap::new();

        if self.title.trim().is_empty() {
            fields.insert("title".to_string(), "is required".to_string());
        } else if self.title.len() > 255 {
            fields.insert(
                "title".to_string(),
                "must be at most 255 characters".to_string(),
            );
        }

        if let Some(ref priority) = self.priority {
            if !["low", "medium", "high"].contains(&priority.as_str()) {
                fields.insert(
                    "priority".to_string(),
                    "must be one of: low, medium, high".to_string(),
                );
            }
        }

        if !fields.is_empty() {
            return Err(AppError::ValidationError { fields });
        }

        Ok(())
    }

    pub fn get_priority(&self) -> String {
        self.priority
            .clone()
            .unwrap_or_else(|| "medium".to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Option<uuid::Uuid>>,
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

        if let Some(ref status) = self.status {
            if !["todo", "in_progress", "done"].contains(&status.as_str()) {
                fields.insert(
                    "status".to_string(),
                    "must be one of: todo, in_progress, done".to_string(),
                );
            }
        }

        if let Some(ref priority) = self.priority {
            if !["low", "medium", "high"].contains(&priority.as_str()) {
                fields.insert(
                    "priority".to_string(),
                    "must be one of: low, medium, high".to_string(),
                );
            }
        }

        if !fields.is_empty() {
            return Err(AppError::ValidationError { fields });
        }

        Ok(())
    }
}

// ============ Pagination ============

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl From<PaginationQuery> for crate::storage::queries::project::Pagination {
    fn from(query: PaginationQuery) -> Self {
        Self::new(query.page.unwrap_or(1), query.limit.unwrap_or(10))
    }
}

// ============ Task Filters ============

#[derive(Debug, Deserialize)]
pub struct TaskFilterQuery {
    pub status: Option<String>,
    pub assignee: Option<uuid::Uuid>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl TaskFilterQuery {
    pub fn validate(&self) -> Result<(), AppError> {
        let mut fields = HashMap::new();

        if let Some(ref status) = self.status {
            if !["todo", "in_progress", "done"].contains(&status.as_str()) {
                fields.insert(
                    "status".to_string(),
                    "must be one of: todo, in_progress, done".to_string(),
                );
            }
        }

        if !fields.is_empty() {
            return Err(AppError::ValidationError { fields });
        }

        Ok(())
    }

    pub fn to_filters(&self) -> crate::storage::queries::task::TaskFilters {
        crate::storage::queries::task::TaskFilters {
            status: self.status.clone(),
            assignee_id: self.assignee,
        }
    }

    pub fn to_pagination(&self) -> crate::storage::queries::project::Pagination {
        crate::storage::queries::project::Pagination::new(
            self.page.unwrap_or(1),
            self.limit.unwrap_or(10),
        )
    }
}

// ============ Helper function ============

pub fn validate_request<T: Validate<Context = ()>>(request: &T) -> Result<(), AppError> {
    request.validate_with(&()).map_err(|e| {
        let mut fields = HashMap::new();
        for (path, error) in e.iter() {
            fields.insert(path.to_string(), error.to_string());
        }
        AppError::ValidationError { fields }
    })
}
