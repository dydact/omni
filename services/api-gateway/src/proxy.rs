use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use std::time::Duration;
use tracing::{error, info};

use crate::config::Config;

#[derive(Clone)]
pub struct ProxyClient {
    client: Client,
    config: Config,
}

impl ProxyClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn forward_request(
        &self,
        method: Method,
        uri: Uri,
        headers: HeaderMap,
        body: Vec<u8>,
        target_service: TargetService,
    ) -> Result<Response, StatusCode> {
        let target_url = match target_service {
            TargetService::Searcher => &self.config.services.searcher_url,
            TargetService::Indexer => &self.config.services.indexer_url,
            TargetService::AI => &self.config.services.ai_service_url,
        };

        let url = format!("{}{}", target_url, uri.path_and_query().map_or("", |pq| pq.as_str()));
        
        info!("Proxying {} {} to {}", method, uri, url);

        let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes())
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let mut request_builder = self.client.request(reqwest_method, &url);

        for (name, value) in headers {
            if let Some(name) = name {
                if let Ok(value_str) = value.to_str() {
                    request_builder = request_builder.header(name.as_str(), value_str);
                }
            }
        }

        if !body.is_empty() {
            request_builder = request_builder.body(body);
        }

        match request_builder.send().await {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();
                
                match response.bytes().await {
                    Ok(body) => {
                        let axum_status = StatusCode::from_u16(status.as_u16())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                        let mut builder = Response::builder().status(axum_status);
                        
                        for (name, value) in headers {
                            if let Some(name) = name {
                                if let Ok(value_str) = value.to_str() {
                                    builder = builder.header(name.as_str(), value_str);
                                }
                            }
                        }

                        builder
                            .body(Body::from(body))
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                    }
                    Err(e) => {
                        error!("Failed to read response body: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
            Err(e) => {
                error!("Failed to forward request to {}: {}", url, e);
                Err(StatusCode::BAD_GATEWAY)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TargetService {
    Searcher,
    Indexer,
    AI,
}