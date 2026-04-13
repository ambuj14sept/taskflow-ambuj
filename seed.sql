-- Seed data for TaskFlow
-- Run this after migrations: docker compose exec postgres psql -U taskflow -d taskflow -f /app/seed.sql

-- Password for test@example.com is 'password123' (bcrypt cost 12)
-- Hash generated with: bcrypt.hash("password123", 12)
-- Note: This is a pre-computed hash for the password 'password123'
INSERT INTO users (id, name, email, password, created_at) VALUES (
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'Test User',
    'test@example.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.flLWrYLYfCDVuS',
    NOW()
) ON CONFLICT (email) DO NOTHING;

INSERT INTO projects (id, name, description, owner_id, created_at) VALUES (
    'b2c3d4e5-f6a7-8901-bcde-f12345678901',
    'Sample Project',
    'A sample project for testing TaskFlow API',
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    NOW()
) ON CONFLICT DO NOTHING;

INSERT INTO tasks (id, title, description, status, priority, project_id, assignee_id, creator_id, due_date, created_at, updated_at) VALUES
    (
        'c3d4e5f6-a7b8-9012-cdef-123456789012',
        'Design homepage',
        'Create wireframes and mockups for the new homepage design',
        'todo',
        'high',
        'b2c3d4e5-f6a7-8901-bcde-f12345678901',
        'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
        'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
        '2026-05-01',
        NOW(),
        NOW()
    ),
    (
        'd4e5f6a7-b8c9-0123-defa-234567890123',
        'Implement authentication',
        'Set up JWT authentication with bcrypt password hashing',
        'in_progress',
        'medium',
        'b2c3d4e5-f6a7-8901-bcde-f12345678901',
        'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
        'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
        '2026-04-20',
        NOW(),
        NOW()
    ),
    (
        'e5f6a7b8-c9d0-1234-efab-345678901234',
        'Write integration tests',
        NULL,
        'done',
        'low',
        'b2c3d4e5-f6a7-8901-bcde-f12345678901',
        NULL,
        'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
        NULL,
        NOW(),
        NOW()
    )
ON CONFLICT DO NOTHING;

-- Test credentials:
-- Email:    test@example.com
-- Password: password123
