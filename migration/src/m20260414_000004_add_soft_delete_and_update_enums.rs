use sea_orm_migration::prelude::*;

use crate::m20260414_000002_create_projects_table::Projects;
use crate::m20260414_000003_create_tasks_table::Tasks;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add is_active to projects
        manager
            .alter_table(
                Table::alter()
                    .table(Projects::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("is_active"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        // Add is_active to tasks
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("is_active"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        // Drop low-cardinality status index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_tasks_status")
                    .table(Tasks::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-create status index
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_status")
                    .table(Tasks::Table)
                    .col(Tasks::Status)
                    .to_owned(),
            )
            .await?;

        // Remove is_active from tasks
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .drop_column(Alias::new("is_active"))
                    .to_owned(),
            )
            .await?;

        // Remove is_active from projects
        manager
            .alter_table(
                Table::alter()
                    .table(Projects::Table)
                    .drop_column(Alias::new("is_active"))
                    .to_owned(),
            )
            .await
    }
}
