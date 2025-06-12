use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn get_current_user() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn login() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn logout() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}