---
source: Context7 API + crates.io
library: SeaORM
package: sea-orm
topic: PostgreSQL setup with tokio + rustls, workspace configuration
fetched: 2026-04-14T00:00:00Z
official_docs: https://www.sea-ql.org/SeaORM/docs/install-and-config/database-and-async-runtime
---

# SeaORM — Setup with PostgreSQL + Tokio + Rustls

## Version Info

| Crate | Latest Stable | Latest Pre-release |
|---|---|---|
| `sea-orm` | **1.1.20** | 2.0.0-rc.38 |
| `sea-orm-migration` | **1.1.20** | 2.0.0-rc.38 |

## Feature Flags for PostgreSQL + Tokio + Rustls

```toml
[dependencies]
sea-orm = { version = "1.1.20", features = [
    "sqlx-postgres",           # PostgreSQL via SQLx
    "runtime-tokio-rustls",    # Tokio async runtime + rustls TLS
    "macros",                  # Required for DeriveEntityModel, etc.
] }
```

### Feature Flag Pattern

The feature flags follow the pattern:
- **DATABASE_DRIVER**: `sqlx-postgres`, `sqlx-mysql`, `sqlx-sqlite`
- **ASYNC_RUNTIME**: `runtime-tokio-native-tls`, `runtime-tokio-rustls`, or `runtime-tokio` (SQLite only)
  - Pattern: `runtime-ASYNC_RUNTIME[-TLS_LIB]`
  - `native-tls`: uses system's native security features
  - `rustls`: nearly pure Rust TLS implementation
  - Note: `async-std` is deprecated; use `tokio`

## Workspace Setup with sea-orm-migration

### Root `Cargo.toml`

```toml
[workspace]
members = [".", "migration"]

[dependencies]
migration = { path = "migration" }
sea-orm = { version = "1.1.20", features = [
    "sqlx-postgres",
    "runtime-tokio-rustls",
    "macros",
] }
```

### Migration Crate `migration/Cargo.toml`

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[dependencies.sea-orm-migration]
version = "1.1.20"
features = [
    # Enable these if you want to run migration via CLI
    "runtime-tokio-rustls",
    "sqlx-postgres",
]
```

### Directory Structure

```
my-project/
├── Cargo.toml              # Workspace root
├── src/
│   └── main.rs             # Application code
└── migration/
    ├── Cargo.toml
    ├── README.md
    └── src/
        ├── lib.rs           # Migrator API (for integration)
        ├── main.rs          # Migrator CLI (for running manually)
        └── m20220101_000001_create_table.rs  # Migration files
```

## Database Connection

```rust
use sea_orm::{Database, DatabaseConnection};

let db: DatabaseConnection = Database::connect("postgres://user:pass@localhost/dbname").await?;
```

## Entity Model Example

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

impl ActiveModelBehavior for ActiveModel {}
```
