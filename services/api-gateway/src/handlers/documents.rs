use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn get_document() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn create_document() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn update_document() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}