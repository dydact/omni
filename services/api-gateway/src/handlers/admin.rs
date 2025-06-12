use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn system_health() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn metrics() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}