# API Gateway Implementation Plan

## Overview
The API Gateway serves as the single entry point for the Clio search platform, handling authentication, authorization, request routing, and providing a unified REST API for the frontend and external clients.

## Core Responsibilities

### 1. Authentication & Authorization
- **Session-Based Auth**: Use `tower-sessions` + `axum-login` for battle-tested session management
- **User Session Management**: Handle login/logout flows with secure cookie-based sessions
- **Role-Based Access Control (RBAC)**: Enforce user permissions via auth traits
- **OAuth Integration**: Support for Google/Slack/Atlassian OAuth flows

### 2. Request Routing & Proxying
- **Service Discovery**: Route requests to appropriate backend services
- **Load Balancing**: Distribute requests across service instances
- **Health Checks**: Monitor backend service availability
- **Circuit Breaker**: Handle service failures gracefully

### 3. Security & Rate Limiting
- **CORS Handling**: Proper cross-origin request support
- **Security Headers**: Implement security best practices
- **Rate Limiting**: Per-user and per-endpoint rate limits
- **Request Validation**: Input sanitization and validation

### 4. API Orchestration
- **Response Aggregation**: Combine responses from multiple services
- **Request Transformation**: Standardize request/response formats
- **Error Handling**: Unified error responses
- **Logging & Monitoring**: Request tracing and metrics

## Architecture Design

### Service Configuration
- **Port**: 3000 (external), 8080 (internal Docker)
- **Framework**: Axum with Tower middleware
- **Database**: PostgreSQL (via shared repository layer)
- **Cache**: Redis for sessions and rate limiting

### Middleware Stack (Order Matters)
1. **CORS Middleware** - Handle cross-origin requests
2. **Security Headers** - Add security headers
3. **Request Logging** - Log all incoming requests
4. **Rate Limiting** - Enforce rate limits
5. **Authentication** - Validate session cookies
6. **Authorization** - Check user permissions
7. **Request Routing** - Route to backend services

### Backend Service Integration
```rust
// Service endpoints
const SEARCHER_URL: &str = "http://searcher:3001";
const INDEXER_URL: &str = "http://indexer:3002"; 
const AI_SERVICE_URL: &str = "http://ai:3003";
```

## API Endpoint Design

### Authentication Endpoints
```
POST /auth/login          - User login with credentials (sets session cookie)
POST /auth/logout         - User logout (clears session cookie)
GET  /auth/me             - Get current user info from session
POST /auth/oauth/google   - Google OAuth callback
POST /auth/oauth/slack    - Slack OAuth callback
```

### Search Endpoints (Proxy to Searcher)
```
POST /api/search          - Unified search across all sources
GET  /api/suggestions     - Search suggestions
GET  /api/sources         - List available data sources
```

### Document Management (Proxy to Indexer)
```
GET    /api/documents/:id - Get document by ID
POST   /api/documents     - Create document
PUT    /api/documents/:id - Update document
DELETE /api/documents/:id - Delete document
```

### AI Features (Proxy to AI Service)
```
POST /api/summarize       - Document summarization
POST /api/embeddings      - Generate embeddings
POST /api/chat            - AI-powered Q&A
```

### Admin Endpoints
```
GET  /api/admin/health    - System health check
GET  /api/admin/metrics   - System metrics
POST /api/admin/sources   - Manage data sources
```

## Implementation Phases

### Phase 1: Core Infrastructure (Priority 1)
- [ ] Set up Axum server with basic routing
- [ ] Implement middleware stack (CORS, logging, error handling)
- [ ] Add health check endpoint
- [ ] Set up service-to-service HTTP client
- [ ] Basic request proxying to searcher service

### Phase 2: Authentication System (Priority 1)
- [ ] Set up tower-sessions with Redis backend
- [ ] Implement axum-login AuthUser trait for Clio users
- [ ] User login/logout endpoints using axum-login patterns
- [ ] Authentication middleware (provided by axum-login)
- [ ] Password hashing with password-auth crate

### Phase 3: Authorization & RBAC (Priority 2)
- [ ] Role-based access control implementation
- [ ] Authorization middleware
- [ ] User permission checking
- [ ] Admin-only endpoints protection

### Phase 4: Service Routing (Priority 2)
- [ ] Complete API endpoint implementation
- [ ] Request/response transformation
- [ ] Error handling and standardization
- [ ] Service health monitoring

### Phase 5: Advanced Features (Priority 3)
- [ ] Rate limiting implementation
- [ ] OAuth integration (Google, Slack, Atlassian)
- [ ] Circuit breaker for service failures
- [ ] Request/response caching
- [ ] Metrics and monitoring

## Technology Stack

### Core Dependencies
```toml
[dependencies]
axum = "0.7"
tower = "0.4"
tower-http = "0.5"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"

# HTTP Client
reqwest = { version = "0.11", features = ["json"] }

# Authentication (Battle-tested crates)
tower-sessions = { version = "0.12", features = ["redis-store"] }
axum-login = "0.15"
bcrypt = "0.15"
password-auth = "1.0"  # Secure password hashing with good defaults

# Database (via shared crate)
shared = { path = "../../shared" }

# Redis
redis = { version = "0.24", features = ["tokio-comp"] }

# Rate Limiting
tower-governor = "0.1"

# CORS
tower-http = { version = "0.5", features = ["cors"] }

# Tracing
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Authentication Architecture Using Proven Crates

```rust
// User type implementing axum-login's AuthUser trait
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClioUser {
    pub id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[async_trait]
impl AuthUser for ClioUser {
    type Id = Uuid;
    
    fn id(&self) -> Self::Id {
        self.id
    }
    
    fn session_auth_hash(&self) -> &[u8] {
        // Used for session invalidation on password change
        self.email.as_bytes()
    }
}

// Auth session type
type AuthSession = axum_login::AuthSession<ClioAuthBackend>;
```

### Configuration Structure
```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub session: SessionConfig,
    pub services: ServiceConfig,
    pub rate_limiting: RateLimitConfig,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub secret_key: String,
    pub ttl: Duration,
    pub cookie_name: String,
    pub cookie_domain: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub searcher_url: String,
    pub indexer_url: String,
    pub ai_service_url: String,
}
```

## Security Considerations

### Session-Based Authentication (Using Proven Crates)

**Primary Crates**:
- **tower-sessions**: Production-ready session management with Redis backend
- **axum-login**: High-level auth patterns, user loading, and role management
- **password-auth**: Secure password hashing with automatic salt generation

**Security Features** (handled by crates):
- **Session Storage**: Redis with automatic TTL management
- **Cookie Security**: HttpOnly, Secure, SameSite=Lax (configured by tower-sessions)
- **Session Data**: Structured session store with type-safe access
- **Session Rotation**: Automatic session ID rotation on auth state changes
- **CSRF Protection**: Built into axum-login
- **Timing Attack Protection**: Constant-time comparisons in password-auth

### Rate Limiting Strategy
- **Per User**: 100 requests/minute for authenticated users
- **Per IP**: 20 requests/minute for unauthenticated requests
- **Per Endpoint**: Search (10/min), Admin (5/min)
- **Storage**: Redis with sliding window

### CORS Configuration
```rust
CorsLayer::new()
    .allow_origin("http://localhost:5173".parse().unwrap()) // SvelteKit dev
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::CONTENT_TYPE, header::COOKIE])
    .allow_credentials(true)
```

## Testing Strategy

### Unit Tests
- Authentication logic (session validation, password hashing)
- Authorization middleware
- Request routing logic
- Rate limiting functionality

### Integration Tests
- End-to-end API flows
- Service-to-service communication
- Database operations via shared repositories
- Redis session management

### Load Testing
- Rate limiting under high load
- Service proxy performance
- Authentication throughput
- Database connection pooling

## Monitoring & Observability

### Logging
- Structured logging with tracing
- Request/response logging
- Error tracking with context
- Performance metrics

### Health Checks
- Service availability monitoring
- Database connection health
- Redis connectivity
- Backend service status

### Metrics
- Request count and latency
- Authentication success/failure rates
- Rate limiting hits
- Service response times

## Deployment Configuration

### Docker Integration
- Update existing docker-compose.yml
- Environment variable configuration
- Health check implementation
- Proper service networking

### Environment Variables
```bash
# Server
API_GATEWAY_HOST=0.0.0.0
API_GATEWAY_PORT=8080

# Database (inherited from shared config)
DATABASE_URL=postgresql://user:pass@postgres:5432/clio

# Redis
REDIS_URL=redis://redis:6379

# Session
SESSION_SECRET_KEY=your-32-byte-secret-key
SESSION_TTL=86400  # 24 hours
COOKIE_DOMAIN=localhost

# Backend Services
SEARCHER_URL=http://searcher:3001
INDEXER_URL=http://indexer:3002
AI_SERVICE_URL=http://ai:3003
```

## Success Criteria

### Functional Requirements
- [ ] All API endpoints respond correctly
- [ ] Authentication and authorization work end-to-end
- [ ] Request routing to backend services functions properly
- [ ] Rate limiting prevents abuse
- [ ] CORS allows frontend access

### Performance Requirements
- [ ] Sub-100ms response time for proxied requests
- [ ] Handle 1000+ concurrent connections
- [ ] Session validation under 5ms
- [ ] Database queries under 50ms

### Security Requirements
- [ ] All endpoints properly authenticated/authorized
- [ ] Session cookies securely generated and validated
- [ ] Rate limiting prevents DoS attacks
- [ ] CORS configured securely
- [ ] No sensitive data in logs

## Next Steps

1. **Review and Approve Plan**: Ensure architecture aligns with requirements
2. **Phase 1 Implementation**: Start with core infrastructure and basic routing
3. **Integration Testing**: Test with existing searcher and indexer services
4. **Frontend Integration**: Update SvelteKit app to use new API Gateway
5. **Production Deployment**: Update docker-compose and deployment scripts

This plan provides a comprehensive roadmap for implementing a production-ready API Gateway that serves as the backbone of the Clio search platform.