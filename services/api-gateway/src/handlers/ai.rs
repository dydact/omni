use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn summarize() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn embeddings() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn chat() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}