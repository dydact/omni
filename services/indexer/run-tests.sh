#!/bin/bash

echo "Running indexer integration tests..."
echo "================================="
echo ""
echo "These tests require PostgreSQL and Redis to be running."
echo ""
echo "If tests fail with connection errors, please ensure:"
echo "1. PostgreSQL is running on localhost:5432"
echo "2. Redis is running on localhost:6379"
echo ""
echo "You can start them with:"
echo "  docker run -d --name test-postgres -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:17"
echo "  docker run -d --name test-redis -p 6379:6379 redis:7-alpine"
echo ""
echo "Running tests..."
echo ""

# Set test environment variables if not already set
export DATABASE_URL=${DATABASE_URL:-"postgresql://postgres:postgres@localhost:5432/postgres"}
export REDIS_URL=${REDIS_URL:-"redis://localhost:6379"}

# Run tests sequentially to avoid database conflicts
cargo test -- --test-threads=1 --nocapture