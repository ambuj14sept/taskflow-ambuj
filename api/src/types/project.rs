use std::collections::HashMap;

use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::storage::entities::project;
use crate::types::common::PaginationMeta;
use crate::types::task::TaskResponse;

/// POST /projects request body
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[garde(length(min = 1, max = 255))]
    pub name: String,

    #[garde(skip)]
    pub description: Option<String>,
}

/// PATCH /projects/:id request body
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

/// Single project in responses
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<project::Model> for ProjectResponse {
    fn from(model: project::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            owner_id: model.owner_id,
            created_at: model.created_at,
        }
    }
}

/// GET /projects response with pagination
#[derive(Debug, Serialize)]
pub struct PaginatedProjectsResponse {
    pub projects: Vec<ProjectResponse>,
    pub pagination: PaginationMeta,
}

/// GET /projects/:id response with tasks
#[derive(Debug, Serialize)]
pub struct ProjectDetailResponse {
    #[serde(flatten)]
    pub project: ProjectResponse,
    pub tasks: Vec<TaskResponse>,
}

/// GET /projects/:id/stats response
#[derive(Debug, Serialize)]
pub struct ProjectStats {
    pub by_status: HashMap<String, u64>,
    pub by_assignee: HashMap<String, u64>,
    pub total: u64,
}
