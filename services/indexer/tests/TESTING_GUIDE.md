# Indexer Service Testing Guide

This directory contains integration tests for the indexer service that test the full flow including database operations.

## Test Structure

```
tests/
├── common/
│   └── mod.rs          # Shared test utilities and fixtures
├── api_integration_test.rs    # REST API endpoint tests
├── event_processor_test.rs    # Redis Pub/Sub event processing tests
├── full_flow_test.rs         # Complete end-to-end flow test
├── README.md                 # Basic setup instructions
└── TESTING_GUIDE.md         # This file
```

## Test Philosophy

These tests follow the principle of **integration over unit testing**:
- Real database connections (PostgreSQL)
- Real message broker (Redis)
- Full application context
- No mocking of external dependencies
- Each test creates isolated test databases

## Key Features

### 1. Database Isolation
Each test creates its own temporary database with a unique name, runs migrations, and cleans up after completion.

### 2. Test Utilities (common/mod.rs)
- `setup_test_app()` - Creates test application with database and Redis
- `setup_test_database()` - Creates isolated test database with migrations
- `setup_test_redis()` - Connects to Redis and flushes test data
- `cleanup_test_database()` - Removes test database after test completion

### 3. Test Fixtures
Pre-configured test data generators for:
- Document creation requests
- Document update requests
- Connector events

## Test Categories

### API Integration Tests
Tests all REST endpoints with real database:
- Health check endpoint
- Document CRUD operations
- Bulk operations
- Concurrent operations
- Error handling

### Event Processor Tests
Tests Redis Pub/Sub event handling:
- Document creation via events
- Document updates via events
- Document deletion via events
- Multiple concurrent events
- Invalid event handling

### Full Flow Test
End-to-end test combining:
- Event-based document creation
- REST API queries
- Mixed updates (REST and events)
- Bulk operations
- Event-based deletion

## Running Tests

```bash
# Prerequisites - start required services
docker run -d --name test-postgres -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:17
docker run -d --name test-redis -p 6379:6379 redis:7-alpine

# Run all tests (sequential to avoid conflicts)
cargo test -- --test-threads=1

# Run specific test file
cargo test --test api_integration_test

# Run with logging
RUST_LOG=info cargo test -- --nocapture

# Use the convenience script
./run-tests.sh
```

## Environment Variables

Tests use these environment variables (with defaults):
- `DATABASE_URL` - PostgreSQL connection (default: postgresql://postgres:postgres@localhost:5432/postgres)
- `REDIS_URL` - Redis connection (default: redis://localhost:6379)
- `RUST_LOG` - Log level for debugging

## Best Practices

1. **Test Independence**: Each test is completely independent
2. **Resource Cleanup**: Always clean up test databases
3. **Real Dependencies**: Use actual PostgreSQL and Redis, not mocks
4. **Sequential Execution**: Run tests with `--test-threads=1` to avoid port conflicts
5. **Descriptive Names**: Test names clearly describe what they test
6. **Error Scenarios**: Include tests for error conditions

## Adding New Tests

1. Create a new test file in the `tests/` directory
2. Import common utilities: `mod common;`
3. Use `setup_test_app()` to get test context
4. Write your test logic
5. Always call `cleanup_test_database()` at the end

Example:
```rust
#[tokio::test]
async fn test_my_feature() {
    let (state, app) = common::setup_test_app().await.unwrap();
    let server = axum_test::TestServer::new(app).unwrap();
    
    // Your test logic here
    
    common::cleanup_test_database(&state.db_pool).await.unwrap();
}
```

## Debugging Failed Tests

1. Enable logging: `RUST_LOG=debug cargo test -- --nocapture`
2. Check database connectivity
3. Verify Redis is running
4. Look for test database cleanup issues
5. Check for port conflicts if running parallel tests

## CI/CD Integration

For CI environments:
1. Use Docker services for PostgreSQL and Redis
2. Set appropriate environment variables
3. Run tests sequentially
4. Ensure proper cleanup even on test failure