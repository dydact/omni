use anyhow::Result;
use indexer::{AppState, create_app};
use redis::Client as RedisClient;
use shared::db::pool::DatabasePool;
use shared::db::repositories::DocumentRepository;
use shared::models::Document;
use sqlx::PgPool;
use std::env;
use tokio::time::{sleep, timeout, Duration};
use uuid::Uuid;

pub async fn setup_test_app() -> Result<(AppState, axum::Router)> {
    let db_pool = setup_test_database().await?;
    let redis_client = setup_test_redis().await?;
    
    let app_state = AppState {
        db_pool,
        redis_client,
    };
    
    let app = create_app(app_state.clone());
    
    Ok((app_state, app))
}

pub async fn setup_test_database() -> Result<DatabasePool> {
    dotenvy::dotenv().ok();
    
    let base_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://clio:clio_password@localhost:5432/clio".to_string());
    
    let test_db_name = format!("clio_test_{}", Uuid::new_v4().to_string().replace("-", ""));
    
    let (base_url_without_db, _) = base_url.rsplit_once('/').unwrap();
    let admin_url = format!("{}/postgres", base_url_without_db);
    
    let admin_pool = PgPool::connect(&admin_url).await?;
    sqlx::query(&format!("CREATE DATABASE {}", test_db_name))
        .execute(&admin_pool)
        .await?;
    
    let test_db_url = format!("{}/{}", base_url_without_db, test_db_name);
    env::set_var("DATABASE_URL", &test_db_url);
    
    let db_pool = DatabasePool::new(&test_db_url).await?;
    
    sqlx::migrate!("./migrations")
        .run(db_pool.pool())
        .await?;
    
    seed_test_data(db_pool.pool()).await?;
    
    Ok(db_pool)
}

pub async fn setup_test_redis() -> Result<RedisClient> {
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let client = RedisClient::open(redis_url)?;
    
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("FLUSHDB").query_async::<String>(&mut conn).await?;
    
    Ok(client)
}

async fn seed_test_data(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at)
        VALUES ('01JGF7V3E0Y2R1X8P5Q7W9T4N6', 'test@example.com', 'hash', NOW(), NOW())
        "#,
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        INSERT INTO sources (id, name, source_type, config, created_by, created_at, updated_at)
        VALUES ('01JGF7V3E0Y2R1X8P5Q7W9T4N7', 'Test Source', 'test', '{}', '01JGF7V3E0Y2R1X8P5Q7W9T4N6', NOW(), NOW())
        "#,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn cleanup_test_database(db_pool: &DatabasePool) -> Result<()> {
    let db_url = env::var("DATABASE_URL")?;
    let (base_url_without_db, db_name) = db_url.rsplit_once('/').unwrap();
    
    // Close all connections to the test database
    db_pool.pool().close().await;
    
    let admin_url = format!("{}/postgres", base_url_without_db);
    let admin_pool = PgPool::connect(&admin_url).await?;
    
    sqlx::query(&format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", db_name))
        .execute(&admin_pool)
        .await?;
    
    Ok(())
}

pub mod fixtures {
    use indexer::{CreateDocumentRequest, UpdateDocumentRequest};
    use serde_json::json;
    
    pub fn create_document_request() -> CreateDocumentRequest {
        CreateDocumentRequest {
            source_id: "01JGF7V3E0Y2R1X8P5Q7W9T4N7".to_string(),
            external_id: "ext_123".to_string(),
            title: "Test Document".to_string(),
            content: "This is test content for integration testing.".to_string(),
            metadata: json!({
                "author": "Test Author",
                "type": "document"
            }),
            permissions: json!({
                "users": ["user1", "user2"],
                "groups": ["group1"]
            }),
        }
    }
    
    pub fn update_document_request() -> UpdateDocumentRequest {
        UpdateDocumentRequest {
            title: Some("Updated Test Document".to_string()),
            content: Some("This is updated content.".to_string()),
            metadata: Some(json!({
                "author": "Updated Author",
                "type": "document",
                "version": 2
            })),
            permissions: Some(json!({
                "users": ["user1", "user2", "user3"],
                "groups": ["group1", "group2"]
            })),
        }
    }
}

/// Wait for a document to exist in the database with polling and timeout
pub async fn wait_for_document_exists(
    repo: &DocumentRepository,
    source_id: &str,
    doc_id: &str,
    timeout_duration: Duration,
) -> Result<Document, String> {
    let result = timeout(timeout_duration, async {
        loop {
            if let Ok(Some(doc)) = repo.find_by_external_id(source_id, doc_id).await {
                return doc;
            }
            sleep(Duration::from_millis(10)).await;
        }
    }).await;
    
    match result {
        Ok(doc) => Ok(doc),
        Err(_) => Err(format!("Document {}:{} not found within timeout", source_id, doc_id)),
    }
}

/// Wait for a document to be deleted from the database with polling and timeout
pub async fn wait_for_document_deleted(
    repo: &DocumentRepository,
    source_id: &str,
    doc_id: &str,
    timeout_duration: Duration,
) -> Result<(), String> {
    let result = timeout(timeout_duration, async {
        loop {
            if let Ok(None) = repo.find_by_external_id(source_id, doc_id).await {
                return;
            }
            sleep(Duration::from_millis(10)).await;
        }
    }).await;
    
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Document {}:{} was not deleted within timeout", source_id, doc_id)),
    }
}