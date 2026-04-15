# TaskFlow API

A task management system backend built with Rust, Actix-web, PostgreSQL, and Redis.

## Overview

TaskFlow is a REST API that allows users to:
- Register and authenticate (JWT + Redis sessions)
- Create and manage projects
- Add tasks to projects with priority and status tracking
- Assign tasks to users
- Filter and paginate task listings
- View project-level statistics (task counts by status and assignee)

## Tech Stack

| Component | Technology | Why |
|-----------|------------|-----|
| Language | Rust (Edition 2024) | Memory-safe, high-performance, strong type system |
| Web Framework | Actix-web 4 | Mature async web framework, battle-tested in production |
| ORM | SeaORM 1 | Async ORM with compile-time query checking |
| Database | PostgreSQL 16 | Robust relational database with strong ACID guarantees |
| Cache/Sessions | Redis 7 | In-memory store for instant session validation and logout |
| Auth | JWT (jsonwebtoken) + bcrypt | Industry-standard token auth with secure password hashing |
| Validation | garde | Derive-macro based request validation with custom rules |
| Logging | tracing + tracing-subscriber | Structured JSON logging with async support |
| Migrations | sea-orm-migration | Rust-native, type-checked, reversible database migrations |

## Architecture Decisions

### Why Redis Sessions (not JWT-only)
Pure JWT auth has one major problem: you cannot invalidate a token before it expires. If a user logs out, their JWT is still valid for up to 24 hours. With Redis-backed sessions, logout = delete Redis key = instant invalidation. No token blacklists or refresh token rotation needed.

### Why SeaORM
SeaORM provides async database access with compile-time query checking. Its migration system (`sea-orm-migration`) produces Rust structs for each migration — type-checked at compile time, with both UP and DOWN directions for full reversibility.

### Why Raw Redis Crate
The raw `redis` crate with `MultiplexedConnection` is sufficient for our session workload (GET/SET/DEL with TTL). A connection pool (`deadpool-redis`) would add complexity without meaningful benefit at this scale.

### Why SCREAMING_SNAKE_CASE for Enums
All enum values (status: `TODO`, `IN_PROGRESS`, `DONE` / priority: `LOW`, `MEDIUM`, `HIGH`) use SCREAMING_SNAKE_CASE consistently across API requests, API responses, and database storage. This is enforced via `strum` derive macros, eliminating manual string conversion and preventing casing inconsistencies.

### Why Soft Delete
Projects and tasks use soft delete (`is_active = false`) instead of hard delete. This preserves data for audit trails, allows recovery of accidentally deleted items, and avoids foreign key cascade issues. All queries filter by `is_active = true` automatically.

### Project Structure
```
taskflow-ambuj/
├── api/                        # Main API crate
│   ├── src/
│   │   ├── main.rs             # Server bootstrap, middleware stack, graceful shutdown
│   │   ├── config/
│   │   │   ├── settings.rs     # Config struct loaded from environment variables
│   │   │   └── global_state.rs # AppState (DB pool, Redis connection, config Arc)
│   │   ├── types/              # Request/response DTOs + shared types
│   │   │   ├── auth.rs         # RegisterRequest, LoginRequest, AuthResponse
│   │   │   ├── project.rs      # Create/Update/Detail/Stats project DTOs
│   │   │   ├── task.rs         # Create/Update/Filter/List task DTOs
│   │   │   ├── enums.rs        # TaskStatus, TaskPriority (strum + serde SCREAMING_SNAKE_CASE)
│   │   │   └── common.rs       # Pagination, PaginationMeta, validate_request helper
│   │   ├── routes/             # HTTP handlers (auth, projects, tasks)
│   │   ├── storage/
│   │   │   ├── entities/       # SeaORM entity models (user, project, task)
│   │   │   └── queries/        # Database operations (user, project, task, access)
│   │   │       └── access.rs   # Project access checks (owner, assignee, creator)
│   │   ├── middleware/
│   │   │   ├── auth.rs         # JWT + Redis session (async FromRequest extractor)
│   │   │   └── request_context.rs # Request-scoped context (request ID, session ID)
│   │   ├── errors/             # AppError enum → HTTP status code mapping
│   │   ├── logging/
│   │   │   ├── formatter.rs    # Structured JSON log formatter, request counter
│   │   │   └── types.rs        # LogEntry, Category (DB/Redis/API/System), Level
│   │   └── validation/         # Placeholder for future custom validators
│   └── tests/                  # Integration tests (29 tests)
├── migration/                  # SeaORM migrations (users, projects, tasks, soft delete)
├── docker-compose.yml          # PostgreSQL + Redis + API (zero-config)
├── Dockerfile                  # Multi-stage build (~25MB final image)
├── seed.sql                    # Test data (1 user, 1 project, 3 tasks)
└── postman/                    # Postman collection for API testing
```

## Running Locally

### Prerequisites
- Docker and Docker Compose
- Git

### Quick Start

```bash
# Clone the repository
git clone https://github.com/ambuj14sept/taskflow-ambuj.git
cd taskflow-ambuj

# Start all services (no .env file needed — all config is in docker-compose.yml)
docker compose up --build -d

# Wait for services to be healthy
sleep 10

# Seed test data
cat seed.sql | docker exec -i taskflow-postgres psql -U taskflow -d taskflow

# Verify the API is running
curl -s http://localhost:9090/health
# Expected: {"status":"healthy"}
```

The API will be available at **http://localhost:9090**

> **Note:** If port 9090 is taken on your machine, change the left side of the port mapping in `docker-compose.yml` (e.g., `"3000:9090"`) and access the API on that port instead.

All environment variables are configured inline in `docker-compose.yml` — no manual setup required. `docker compose up --build -d` handles everything:
1. Pulls PostgreSQL and Redis images
2. Builds the Rust API (multi-stage Docker build)
3. Starts PostgreSQL and Redis, waits for health checks
4. Starts the API, runs database migrations automatically
5. API ready to accept requests

### Testing the API

Import the Postman collection from `postman/taskflow.postman_collection.json` into Postman:
1. Open Postman → Import → Upload File → select `postman/taskflow.postman_collection.json`
2. The `base_url` variable is set to `http://localhost:9090` — change it if you used a different port
3. Run **Login** or **Register** first — the token is captured automatically and used by all other requests

Or use curl:

```bash
# Register a new user
curl -s -X POST http://localhost:9090/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"Jane Doe","email":"jane@example.com","password":"secret123"}'

# Login and save the token
TOKEN=$(curl -s -X POST http://localhost:9090/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}' | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])")

# Create a project
curl -s -X POST http://localhost:9090/projects \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"My Project","description":"A test project"}'

# List projects
curl -s http://localhost:9090/projects \
  -H "Authorization: Bearer $TOKEN"

# Create a task (replace PROJECT_ID with the id from create project response)
curl -s -X POST http://localhost:9090/projects/PROJECT_ID/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"My first task","priority":"HIGH","due_date":"2026-05-01"}'

# Logout
curl -s -X POST http://localhost:9090/auth/logout \
  -H "Authorization: Bearer $TOKEN"
```

### Environment Variables

For local development outside Docker, copy `.env.example`:

```bash
cp .env.example .env
cargo run -p api
```

| Variable | Description | Default |
|----------|-------------|---------|
| `DB_HOST` | PostgreSQL host | `postgres` |
| `DB_PORT` | PostgreSQL port | `5432` |
| `DB_USER` | Database user | `taskflow` |
| `DB_PASSWORD` | Database password | `taskflow_secret` |
| `DB_NAME` | Database name | `taskflow` |
| `DB_POOL_SIZE` | Connection pool size | `10` |
| `REDIS_HOST` | Redis host | `redis` |
| `REDIS_PORT` | Redis port | `6379` |
| `REDIS_PASSWORD` | Redis password (empty = no auth) | *(empty)* |
| `REDIS_DB` | Redis database number | `0` |
| `JWT_SECRET` | JWT signing secret | **Required** |
| `JWT_EXPIRY_HOURS` | Token validity (hours) | `24` |
| `BCRYPT_COST` | Password hash cost factor | `12` |
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `9090` |
| `ENV` | Environment name | `dev` |

## Database Migrations

Migrations run **automatically** on container startup via `Migrator::up()` in `AppState::new()`. No manual steps required.

Four migration files create and evolve the schema:
1. `m20260414_000001_create_users_table` — users with bcrypt passwords
2. `m20260414_000002_create_projects_table` — projects with owner FK
3. `m20260414_000003_create_tasks_table` — tasks with status, priority, assignee, creator FKs + indexes
4. `m20260414_000004_add_soft_delete_and_update_enums` — adds `is_active` column to projects and tasks, drops low-cardinality status index

All migrations have both UP (create) and DOWN (drop) directions for full reversibility.

## Test Credentials

Seed data is included for immediate testing:

```
Email:    test@example.com
Password: password123
```

To load seed data:
```bash
cat seed.sql | docker exec -i taskflow-postgres psql -U taskflow -d taskflow
```

This creates 1 user, 1 project, and 3 tasks (todo, in_progress, done).

## Running Tests

The project includes **29 integration tests** covering auth, projects, and tasks.

### Prerequisites

1. **PostgreSQL** running and accessible from your machine
2. **Redis** running and accessible from your machine
3. A `.env.test` file in the project root configured to point to your PostgreSQL and Redis instances

Tests connect directly to PostgreSQL and Redis — no Docker is required. As long as both services are reachable at the host/port specified in `.env.test`, the tests will work.

> **Note:** Migrations run automatically when the test `AppState` is created — no manual schema setup needed. The database schema is created by `Migrator::up()` in `AppState::new()`.

### How Test Configuration Works

Tests load environment variables from `.env.test` only:

```rust
dotenvy::from_filename(".env.test").ok();
```

`.env.test` is a self-contained config file for tests — it does not depend on `.env`. Just make sure the `DB_HOST`, `DB_PORT`, `REDIS_HOST`, and `REDIS_PORT` values point to your running PostgreSQL and Redis instances.

The `.env.test` file isolates test data by using `REDIS_DB=1` (separate Redis keyspace from the running API) and a lower `BCRYPT_COST=4` (faster hashing in tests).

If your PostgreSQL or Redis runs on non-default ports, just update `.env.test`:
```bash
DB_PORT=5434       # Your PostgreSQL port
REDIS_PORT=6380    # Your Redis port
```

### Running All Tests

```bash
cargo test --package api -- --test-threads=1
```

### Running a Specific Test Suite

```bash
# Auth tests only
cargo test --package api --test auth_tests -- --test-threads=1

# Project tests only
cargo test --package api --test project_tests -- --test-threads=1

# Task tests only
cargo test --package api --test task_tests -- --test-threads=1
```

### Why `--test-threads=1`?

Tests share a single PostgreSQL database and Redis instance. Running them sequentially prevents race conditions (e.g., two tests inserting the same email simultaneously).

### Test Coverage

| Suite | Tests | What's Covered |
|-------|-------|----------------|
| **Auth** | 9 | Register (success, validation, duplicate email), Login (success, wrong password, nonexistent), Logout (session invalidation), Protected routes (no token, invalid token) |
| **Projects** | 9 | Create (success, validation), List (with pagination), Get detail (with tasks, 404), Update/Delete (owner-only → 403), Stats (by_status, by_assignee) |
| **Tasks** | 11 | Create (success, defaults, validation), List (status filter, pagination), Update (fields + updated_at, invalid status), Delete (by creator, non-owner rejected), Cascade delete, Invalid filter |

## API Reference

### Authentication (public — no token required)

#### POST /auth/register
```json
// Request
{ "name": "Jane Doe", "email": "jane@example.com", "password": "secret123" }

// Response 201
{ "token": "<jwt>", "user": { "id": "uuid", "name": "Jane Doe", "email": "jane@example.com" } }
```

#### POST /auth/login
```json
// Request
{ "email": "jane@example.com", "password": "secret123" }

// Response 200
{ "token": "<jwt>", "user": { "id": "uuid", "name": "Jane Doe", "email": "jane@example.com" } }
```

#### POST /auth/logout
**Headers:** `Authorization: Bearer <token>`
```json
// Response 200
{ "message": "logged out successfully" }
```

### Projects (all require `Authorization: Bearer <token>`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/projects?page=1&limit=10` | List projects user owns or has tasks in |
| POST | `/projects` | Create project (owner = current user) |
| GET | `/projects/:id` | Project details + all tasks |
| PATCH | `/projects/:id` | Update name/description (owner only → 403) |
| DELETE | `/projects/:id` | Soft delete project + cascade soft-delete tasks (owner only → 403), 404 if not found |
| GET | `/projects/:id/stats` | Task counts by status and assignee |

### Tasks (all require `Authorization: Bearer <token>`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/projects/:id/tasks?status=TODO&assignee=uuid&page=1&limit=10` | List with filters + pagination. Status values: `TODO`, `IN_PROGRESS`, `DONE` |
| POST | `/projects/:id/tasks` | Create task (creator_id = current user, status defaults to `TODO`, priority defaults to `MEDIUM`) |
| PATCH | `/tasks/:id` | Update fields (title, description, status, priority, assignee, due_date). Priority values: `LOW`, `MEDIUM`, `HIGH` |
| DELETE | `/tasks/:id` | Soft delete (project owner or task creator only), 404 if not found or not permitted |

### Error Responses

| Status | When | Response |
|--------|------|----------|
| 400 | Validation failure | `{ "error": "validation failed", "fields": { "email": "not a valid email address" } }` |
| 401 | No token / invalid token / expired session | `{ "error": "unauthorized" }` |
| 401 | Wrong email or password | `{ "error": "invalid email or password" }` |
| 403 | Valid user but not permitted | `{ "error": "forbidden" }` |
| 404 | Resource not found | `{ "error": "not found" }` |
| 409 | Duplicate email | `{ "error": "conflict: email already exists" }` |

## What I'd Do With More Time

1. **Rate Limiting** — Request throttling per user/IP to prevent abuse
2. **Email Verification** — Confirm email ownership before account activation
3. **Password Reset** — Secure password reset flow via email
4. **Task Comments** — Allow users to comment on tasks
5. **WebSocket Notifications** — Real-time updates when tasks change
6. **Audit Logging** — Track all data changes with who/when/what
7. **OpenAPI/Swagger** — Auto-generated API documentation from code
8. **CI/CD Pipeline** — GitHub Actions for automated testing and deployment
9. **Session Management** — List active sessions, "logout from all devices"
10. **Task Attachments** — File uploads for tasks

## License

MIT
