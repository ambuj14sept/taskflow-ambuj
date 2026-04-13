use sea_orm_migration::prelude::*;

use crate::m20260414_000001_create_users_table::Users;
use crate::m20260414_000002_create_projects_table::Projects;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tasks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tasks::Title).string().not_null())
                    .col(ColumnDef::new(Tasks::Description).text())
                    .col(
                        ColumnDef::new(Tasks::Status)
                            .string()
                            .not_null()
                            .default("todo"),
                    )
                    .col(
                        ColumnDef::new(Tasks::Priority)
                            .string()
                            .not_null()
                            .default("medium"),
                    )
                    .col(ColumnDef::new(Tasks::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(Tasks::AssigneeId).uuid())
                    .col(ColumnDef::new(Tasks::CreatorId).uuid().not_null())
                    .col(ColumnDef::new(Tasks::DueDate).date())
                    .col(
                        ColumnDef::new(Tasks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Tasks::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_project_id")
                            .from(Tasks::Table, Tasks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_assignee_id")
                            .from(Tasks::Table, Tasks::AssigneeId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_creator_id")
                            .from(Tasks::Table, Tasks::CreatorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes for common queries
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_project_id")
                    .table(Tasks::Table)
                    .col(Tasks::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_status")
                    .table(Tasks::Table)
                    .col(Tasks::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_assignee_id")
                    .table(Tasks::Table)
                    .col(Tasks::AssigneeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_creator_id")
                    .table(Tasks::Table)
                    .col(Tasks::CreatorId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tasks {
    Table,
    Id,
    Title,
    Description,
    Status,
    Priority,
    ProjectId,
    AssigneeId,
    CreatorId,
    DueDate,
    CreatedAt,
    UpdatedAt,
}
