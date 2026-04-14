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

### Project Structure
```
taskflow-ambuj/
├── api/                        # Main API crate
│   ├── src/
│   │   ├── config/             # Config struct + AppState initialization
│   │   ├── routes/             # HTTP handlers (auth, projects, tasks)
│   │   ├── storage/
│   │   │   ├── entities/       # SeaORM models (user, project, task)
│   │   │   └── queries/        # Database operations per table
│   │   ├── middleware/         # JWT auth (async FromRequest) + request context
│   │   ├── errors/             # AppError enum → HTTP status code mapping
│   │   ├── logging/            # Structured JSON logging with Category enum
│   │   └── validation/         # garde validators + custom validation functions
│   └── tests/                  # Integration tests (29 tests)
├── migration/                  # SeaORM migrations (users, projects, tasks)
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
docker compose up --build
```

The API will be available at **http://localhost:8080**

All environment variables are configured inline in `docker-compose.yml` — no manual setup required. `docker compose up` handles everything:
1. Pulls PostgreSQL and Redis images
2. Builds the Rust API (multi-stage Docker build)
3. Starts PostgreSQL and Redis, waits for health checks
4. Starts the API, runs database migrations automatically
5. API ready to accept requests

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
| `JWT_SECRET` | JWT signing secret | **Required** |
| `JWT_EXPIRY_HOURS` | Token validity (hours) | `24` |
| `BCRYPT_COST` | Password hash cost factor | `12` |
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `8080` |
| `ENV` | Environment name | `dev` |

## Database Migrations

Migrations run **automatically** on container startup via `Migrator::up()` in `AppState::new()`. No manual steps required.

Three migration files create the schema:
1. `m20260414_000001_create_users_table` — users with bcrypt passwords
2. `m20260414_000002_create_projects_table` — projects with owner FK
3. `m20260414_000003_create_tasks_table` — tasks with status, priority, assignee, creator FKs + indexes

All migrations have both UP (create) and DOWN (drop) directions for full reversibility.

## Test Credentials

Seed data is included for immediate testing:

```
Email:    test@example.com
Password: password123
```

To load seed data:
```bash
docker compose exec postgres psql -U taskflow -d taskflow -f /app/seed.sql
```

This creates 1 user, 1 project, and 3 tasks (todo, in_progress, done).

## Running Tests

The project includes **29 integration tests** covering auth, projects, and tasks.

Tests require PostgreSQL and Redis running (from docker compose):

```bash
# Start database and redis
docker compose up postgres redis -d

# Run tests (point to docker-mapped ports)
DB_HOST=localhost DB_PORT=5432 DB_USER=taskflow DB_PASSWORD=taskflow_secret DB_NAME=taskflow \
REDIS_HOST=localhost REDIS_PORT=6379 \
JWT_SECRET=test-secret \
ENV=test \
cargo test --package api -- --test-threads=1
```

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
| DELETE | `/projects/:id` | Delete project + cascade tasks (owner only → 403) |
| GET | `/projects/:id/stats` | Task counts by status and assignee |

### Tasks (all require `Authorization: Bearer <token>`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/projects/:id/tasks?status=todo&assignee=uuid&page=1&limit=10` | List with filters + pagination |
| POST | `/projects/:id/tasks` | Create task (creator_id = current user, status defaults to "todo") |
| PATCH | `/tasks/:id` | Update fields (title, description, status, priority, assignee, due_date) |
| DELETE | `/tasks/:id` | Delete (project owner or task creator only) |

### Error Responses

| Status | When | Response |
|--------|------|----------|
| 400 | Validation failure | `{ "error": "validation failed", "fields": { "email": "invalid email format" } }` |
| 401 | No token / invalid token / expired session | `{ "error": "unauthorized" }` |
| 401 | Wrong email or password | `{ "error": "invalid email or password" }` |
| 403 | Valid user but not permitted | `{ "error": "forbidden" }` |
| 404 | Resource not found | `{ "error": "not found" }` |
| 409 | Duplicate email | `{ "error": "conflict: email already exists" }` |

## Postman Collection

Import `postman/taskflow.postman_collection.json` into Postman for interactive API testing. The collection includes pre-request scripts that automatically capture the JWT token from login/register responses.

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
