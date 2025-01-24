use axum::{
    routing::post,
    Router,
    extract::DefaultBodyLimit,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::application::image_processor::ImageProcessor;
use crate::presentation::handlers;
use crate::domain::{ServerConstants, ErrorMessages};

pub async fn create_app() -> Router {
    let image_processor = Arc::new(
        ImageProcessor::new().expect(ErrorMessages::FAILED_TO_INITIALIZE_IMAGE_PROCESSOR)
    );

    Router::new()
        .route(ServerConstants::PATH_REMOVE_BACKGROUND, post(handlers::remove_background))
        .route(ServerConstants::PATH_BATCH_REMOVE_BACKGROUND, post(handlers::batch_remove_background))
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(ServerConstants::MAX_BODY_SIZE))
        .with_state(image_processor)
} 