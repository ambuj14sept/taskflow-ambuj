# TaskFlow API

A minimal but complete task management system backend built with Rust, Actix-web, and PostgreSQL.

## Overview

TaskFlow is a REST API that allows users to:
- Register and authenticate
- Create and manage projects
- Add tasks to projects
- Assign tasks to users
- Filter and paginate task listings
- View project statistics

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (Edition 2024) |
| Web Framework | Actix-web 4 |
| ORM | SeaORM 1 |
| Database | PostgreSQL 16 |
| Cache/Sessions | Redis 7 |
| Auth | JWT + bcrypt |
| Validation | garde |
| Logging | tracing + tracing-subscriber |

## Architecture Decisions

### Why Actix-web
Actix-web is a mature, high-performance async web framework. It's battle-tested and widely used in production Rust services. Chosen for consistency with existing codebases.

### Why SeaORM
SeaORM provides async database access with compile-time query checking. Its migration system (`sea-orm-migration`) is Rust-native and integrates seamlessly.

### Why Redis Sessions
Pure JWT auth cannot be invalidated before expiry. Redis-backed sessions allow instant logout and session revocation. Session data is minimal (just user_id) for performance.

### Why Raw Redis Crate
The raw `redis` crate with `MultiplexedConnection` is sufficient for this workload. No connection pool needed — multiplexing handles concurrent commands on a single connection.

### Project Structure
```
api/src/
├── config/          # Configuration and global state
├── routes/          # HTTP handlers
├── storage/         # Database entities and queries
│   ├── entities/    # SeaORM models
│   └── queries/     # Database operations
├── middleware/      # Auth and request context
├── errors/          # Error types
├── logging/         # Custom structured logging
└── validation/      # Request validation
```

## Running Locally

### Prerequisites
- Docker and Docker Compose
- Git

### Quick Start

```bash
# Clone the repository
git clone https://github.com/ambujkumar/taskflow-ambuj
cd taskflow-ambuj

# Copy environment file
cp .env.example .env

# Start all services
docker compose up --build
```

The API will be available at http://localhost:8080

### Environment Variables

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
| `JWT_EXPIRY_HOURS` | Token validity | `24` |
| `BCRYPT_COST` | Password hash cost | `12` |
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `8080` |
| `ENV` | Environment name | `dev` |

## Running Migrations

Migrations run automatically on container startup. No manual steps required.

To run manually (for development):
```bash
cargo run -p migration
```

## Test Credentials

```
Email:    test@example.com
Password: password123
```

Run the seed script after migrations:
```bash
docker compose exec postgres psql -U taskflow -d taskflow -f /app/seed.sql
```

## API Reference

### Authentication

#### POST /auth/register
Register a new user.

```json
// Request
{
    "name": "Jane Doe",
    "email": "jane@example.com",
    "password": "secret123"
}

// Response 201
{
    "token": "<jwt>",
    "user": {
        "id": "uuid",
        "name": "Jane Doe",
        "email": "jane@example.com"
    }
}
```

#### POST /auth/login
Authenticate and receive a JWT.

```json
// Request
{
    "email": "jane@example.com",
    "password": "secret123"
}

// Response 200
{
    "token": "<jwt>",
    "user": { "id": "uuid", "name": "Jane Doe", "email": "jane@example.com" }
}
```

#### POST /auth/logout
Invalidate the current session.

**Headers:** `Authorization: Bearer <token>`

```json
// Response 200
{ "message": "logged out successfully" }
```

### Projects

All project endpoints require authentication.

#### GET /projects
List projects the user owns or has tasks in.

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 10, max: 100)

```json
// Response 200
{
    "projects": [
        {
            "id": "uuid",
            "name": "My Project",
            "description": "Project description",
            "owner_id": "uuid",
            "created_at": "2026-04-14T10:00:00Z"
        }
    ],
    "pagination": {
        "page": 1,
        "limit": 10,
        "total": 1,
        "total_pages": 1
    }
}
```

#### POST /projects
Create a new project.

```json
// Request
{
    "name": "New Project",
    "description": "Optional description"
}

// Response 201
{ "id": "uuid", "name": "New Project", ... }
```

#### GET /projects/:id
Get project details with all tasks.

#### PATCH /projects/:id
Update project (owner only).

```json
// Request
{ "name": "Updated Name" }
```

#### DELETE /projects/:id
Delete project and all tasks (owner only).

#### GET /projects/:id/stats
Get task statistics for a project.

```json
// Response 200
{
    "by_status": { "todo": 5, "in_progress": 3, "done": 2 },
    "by_assignee": { "uuid": 4, "unassigned": 6 },
    "total": 10
}
```

### Tasks

#### GET /projects/:id/tasks
List tasks with optional filters.

**Query Parameters:**
- `status` (optional): Filter by status (`todo`, `in_progress`, `done`)
- `assignee` (optional): Filter by assignee UUID
- `page`, `limit`: Pagination

#### POST /projects/:id/tasks
Create a task.

```json
// Request
{
    "title": "Task title",
    "description": "Optional description",
    "priority": "high",
    "assignee_id": "uuid",
    "due_date": "2026-04-30"
}
```

#### PATCH /tasks/:id
Update a task.

```json
// Request
{
    "title": "Updated title",
    "status": "done",
    "priority": "low"
}
```

#### DELETE /tasks/:id
Delete a task (project owner or task creator only).

## Error Responses

```json
// 400 Validation Error
{
    "error": "validation failed",
    "fields": {
        "email": "invalid email format",
        "password": "must be at least 8 characters"
    }
}

// 401 Unauthorized
{ "error": "unauthorized" }

// 403 Forbidden
{ "error": "forbidden" }

// 404 Not Found
{ "error": "not found" }

// 500 Internal Error
{ "error": "internal server error" }
```

## Postman Collection

Import `postman/taskflow.postman_collection.json` into Postman for interactive API testing.

## What I'd Do With More Time

1. **Integration Tests**: Add comprehensive test suite covering all endpoints and edge cases
2. **Rate Limiting**: Implement request rate limiting per user/IP
3. **Email Verification**: Add email confirmation for new accounts
4. **Password Reset**: Implement password reset flow via email
5. **Task Comments**: Allow users to comment on tasks
6. **File Attachments**: Support file uploads for tasks
7. **WebSocket Notifications**: Real-time updates for task changes
8. **Audit Logging**: Track all data changes for compliance
9. **OpenAPI Documentation**: Generate Swagger/OpenAPI spec from code
10. **CI/CD Pipeline**: GitHub Actions for automated testing and deployment

## License

MIT
