use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true)
}

pub fn trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
> {
    TraceLayer::new_for_http()
}