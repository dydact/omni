use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tracing::info;
use tracing_subscriber;

mod config;
mod handlers;
mod middleware;
mod proxy;

pub use proxy::TargetService;

use config::Config;
use proxy::ProxyClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = Config::from_env();
    info!("Starting API Gateway on {}:{}", config.host, config.port);

    let app = create_app(config.clone()).await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("API Gateway listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub proxy: ProxyClient,
}

async fn create_app(config: Config) -> anyhow::Result<Router> {
    let proxy = ProxyClient::new(config.clone());
    let state = AppState { config, proxy };
    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        
        // Authentication routes (placeholder for Phase 2)
        .route("/auth/me", get(handlers::auth::get_current_user))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/logout", post(handlers::auth::logout))
        
        // API routes that will proxy to backend services
        .route("/api/search", post(handlers::search::search))
        .route("/api/suggestions", get(handlers::search::suggestions))
        .route("/api/sources", get(handlers::search::sources))
        
        // Document management (proxy to indexer)
        .route("/api/documents/:id", get(handlers::documents::get_document))
        .route("/api/documents", post(handlers::documents::create_document))
        .route("/api/documents/:id", post(handlers::documents::update_document))
        
        // AI features (proxy to AI service)
        .route("/api/summarize", post(handlers::ai::summarize))
        .route("/api/embeddings", post(handlers::ai::embeddings))
        .route("/api/chat", post(handlers::ai::chat))
        
        // Admin endpoints
        .route("/api/admin/health", get(handlers::admin::system_health))
        .route("/api/admin/metrics", get(handlers::admin::metrics))
        
        // Middleware stack (order matters!)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::trace_layer())
                .layer(middleware::cors_layer())
        )
        .with_state(state);

    Ok(app)
}
