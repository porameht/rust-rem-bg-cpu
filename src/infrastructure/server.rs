use axum::{
    routing::post,
    Router,
    extract::DefaultBodyLimit,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::application::image_processor::ImageProcessor;
use crate::presentation::handlers;

pub async fn create_app() -> Router {
    let image_processor = Arc::new(
        ImageProcessor::new().expect("Failed to initialize image processor")
    );

    Router::new()
        .route("/api/rem-bg", post(handlers::remove_background))
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10)) // 10MB limit
        .with_state(image_processor)
} 