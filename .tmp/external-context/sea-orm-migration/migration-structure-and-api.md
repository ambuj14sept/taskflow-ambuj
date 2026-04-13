---
source: Context7 API + crates.io
library: SeaORM Migration
package: sea-orm-migration
topic: Migration crate structure, MigratorTrait, SchemaManager, up/down migrations
fetched: 2026-04-14T00:00:00Z
official_docs: https://www.sea-ql.org/SeaORM/docs/migration/writing-migration
---

# sea-orm-migration — Migration Crate Structure & API

## Version Info

| Crate | Latest Stable | Latest Pre-release |
|---|---|---|
| `sea-orm-migration` | **1.1.20** | 2.0.0-rc.38 |

## Migration Crate Structure

```
migration/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs                              # Migrator API, for integration
    ├── main.rs                             # Migrator CLI, for running manually
    └── m20220101_000001_create_table.rs    # A sample migration file
```

### Generate New Migration Files

```bash
sea-orm-cli migrate generate NAME_OF_MIGRATION [--local-time]

# E.g. to generate `migration/src/m20220101_000001_create_table.rs`
sea-orm-cli migrate generate create_table

# Equivalent (spaces converted to underscores)
sea-orm-cli migrate generate "create table"
```

File naming convention: `mYYYYMMDD_HHMMSS_migration_name.rs`

## MigratorTrait Implementation (`lib.rs`)

```rust
pub use sea_orm_migration::*;

mod m20220101_000001_create_table;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
        ]
    }
}
```

**Important**: Migrations must be sorted chronologically in the `migrations()` vec.

## Writing Up/Down Migrations with SchemaManager

### Basic Migration Structure

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Post::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Post::Title).string().not_null())
                    .col(ColumnDef::new(Post::Text).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    Text,
}
```

### Using Schema Helper Functions (Concise Syntax)

```rust
use sea_orm_migration::{prelude::*, schema::*};

// Schema helpers: pk_auto, string, string_null, integer, enumeration_null, etc.
manager
    .create_table(
        Table::create()
            .table("post")
            .if_not_exists()
            .col(pk_auto("id"))
            .col(string("title"))
            .col(string("text"))
            .col(enumeration_null("category", "category", ["Feed", "Store"]))
    )
    .await
```

### Combining Multiple Schema Changes

```rust
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.create_table(
        sea_query::Table::create()
            .table(Post::Table)
            .if_not_exists()
            .col(pk_auto(Post::Id))
            .col(string(Post::Title))
            .col(string(Post::Text))
    ).await?;

    manager.create_index(
        Index::create()
            .if_not_exists()
            .name("idx-post_title")
            .table(Post::Table)
            .col(Post::Title)
    ).await?;

    Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_index(Index::drop().name("idx-post-title")).await?;
    manager.drop_table(Table::drop().table(Post::Table)).await?;
    Ok(())
}
```

## SchemaManager API Reference

### Creation Methods
- `create_table(TableCreateStatement)` — Create a new table
- `create_index(IndexCreateStatement)` — Create an index
- `create_foreign_key(ForeignKeyCreateStatement)` — Create a foreign key
- `create_type(TypeCreateStatement)` — Create a custom type (PostgreSQL)

### Mutation Methods
- `drop_table(TableDropStatement)` — Drop a table
- `drop_index(IndexDropStatement)` — Drop an index
- `drop_foreign_key(ForeignKeyDropStatement)` — Drop a foreign key
- `alter_table(TableAlterStatement)` — Alter a table
- `rename_table(TableRenameStatement)` — Rename a table
- `truncate_table(TableTruncateStatement)` — Truncate a table

### Inspection Methods
- `has_table(name)` — Check if a table exists
- `has_column(table, column)` — Check if a column exists
- `has_index(table, index)` — Check if an index exists

## Running Migrations Programmatically

```rust
use migration::{Migrator, MigratorTrait};

// Apply all pending migrations
Migrator::up(db, None).await?;

// Apply 10 pending migrations
Migrator::up(db, Some(10)).await?;

// Rollback all applied migrations
Migrator::down(db, None).await?;

// Rollback last 10 applied migrations
Migrator::down(db, Some(10)).await?;

// Check the status of all migrations
Migrator::status(db).await?;

// Drop all tables, then reapply all migrations
Migrator::fresh(db).await?;

// Rollback all, then reapply all migrations
Migrator::refresh(db).await?;

// Rollback all applied migrations
Migrator::reset(db).await?;
```
