use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    response::Response,
};

use crate::{proxy::TargetService, AppState};

pub async fn search(State(state): State<AppState>, req: Request) -> Result<Response, StatusCode> {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .to_vec();

    state
        .proxy
        .forward_request(
            Method::POST,
            parts.uri,
            parts.headers,
            body_bytes,
            TargetService::Searcher,
        )
        .await
}

pub async fn suggestions(State(state): State<AppState>, req: Request) -> Result<Response, StatusCode> {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .to_vec();

    state
        .proxy
        .forward_request(
            Method::GET,
            parts.uri,
            parts.headers,
            body_bytes,
            TargetService::Searcher,
        )
        .await
}

pub async fn sources(State(state): State<AppState>, req: Request) -> Result<Response, StatusCode> {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .to_vec();

    state
        .proxy
        .forward_request(
            Method::GET,
            parts.uri,
            parts.headers,
            body_bytes,
            TargetService::Searcher,
        )
        .await
}