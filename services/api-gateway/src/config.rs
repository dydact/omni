use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub services: ServiceConfig,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub searcher_url: String,
    pub indexer_url: String,
    pub ai_service_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("API_GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("API_GATEWAY_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("API_GATEWAY_PORT must be a valid port number"),
            services: ServiceConfig {
                searcher_url: env::var("SEARCHER_URL")
                    .unwrap_or_else(|_| "http://searcher:3001".to_string()),
                indexer_url: env::var("INDEXER_URL")
                    .unwrap_or_else(|_| "http://indexer:3002".to_string()),
                ai_service_url: env::var("AI_SERVICE_URL")
                    .unwrap_or_else(|_| "http://ai:3003".to_string()),
            },
        }
    }
}