use searcher::models::{SearchMode, SearchRequest};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "searcher");
}

#[tokio::test]
async fn test_search_empty_query() {
    let app = create_test_app().await;
    
    let request = SearchRequest {
        query: "".to_string(),
        sources: None,
        content_types: None,
        limit: Some(10),
        offset: None,
        mode: Some(SearchMode::Fulltext),
    };
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_search_with_query() {
    let app = create_test_app().await;
    
    let request = SearchRequest {
        query: "test document".to_string(),
        sources: None,
        content_types: None,
        limit: Some(20),
        offset: None,
        mode: Some(SearchMode::Fulltext),
    };
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["results"].is_array());
    assert!(json["query_time_ms"].is_number());
    assert_eq!(json["query"], "test document");
}

#[tokio::test]
async fn test_search_modes() {
    let app = create_test_app().await;
    
    // Test all three search modes
    for mode in [SearchMode::Fulltext, SearchMode::Semantic, SearchMode::Hybrid] {
        let request = SearchRequest {
            query: "test".to_string(),
            sources: None,
            content_types: None,
            limit: Some(5),
            offset: None,
            mode: Some(mode.clone()),
        };
        
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/search")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), 200, "Failed for mode: {:?}", mode);
    }
}

#[tokio::test]
async fn test_suggestions() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/suggestions?q=test&limit=5")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["suggestions"].is_array());
    assert_eq!(json["query"], "test");
}

#[tokio::test]
async fn test_search_with_filters() {
    let app = create_test_app().await;
    
    let request = SearchRequest {
        query: "document".to_string(),
        sources: Some(vec!["source1".to_string(), "source2".to_string()]),
        content_types: Some(vec!["text/plain".to_string()]),
        limit: Some(10),
        offset: Some(5),
        mode: Some(SearchMode::Fulltext),
    };
    
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
}

async fn create_test_app() -> axum::Router {
    use shared::db::DatabasePool;
    
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/clio_test".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let db_pool = DatabasePool::new(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");
    
    searcher::create_app(searcher::AppState {
        db_pool: db_pool.pool().clone(),
        redis_client,
    })
}