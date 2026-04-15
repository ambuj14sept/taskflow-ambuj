pub use sea_orm_migration::prelude::*;

mod m20260414_000001_create_users_table;
mod m20260414_000002_create_projects_table;
mod m20260414_000003_create_tasks_table;
mod m20260414_000004_add_soft_delete_and_update_enums;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260414_000001_create_users_table::Migration),
            Box::new(m20260414_000002_create_projects_table::Migration),
            Box::new(m20260414_000003_create_tasks_table::Migration),
            Box::new(m20260414_000004_add_soft_delete_and_update_enums::Migration),
        ]
    }
}
