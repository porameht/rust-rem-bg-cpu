use axum::{
    extract::{State, Multipart},
    response::Response,
    http::{header, StatusCode},
};
use std::sync::Arc;
use crate::application::image_processor::ImageProcessor;
use crate::domain::{AppError, ServerConstants};
use tracing;

pub async fn remove_background(
    State(processor): State<Arc<ImageProcessor>>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let start_time = std::time::Instant::now();
    tracing::info!("Processing background removal request");
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to process multipart form: {}", e);
        AppError::ImageProcessingError(e.to_string())
    })? {
        if field.name() == Some(ServerConstants::FIELD_IMAGE) {
            let data = field.bytes().await.map_err(|e| {
                tracing::error!("Failed to read image data: {}", e);
                AppError::ImageProcessingError(e.to_string())
            })?;
            
            match processor.remove_background(&data).await {
                Ok(result) => {
                    tracing::info!("Success - took {:.2?}", start_time.elapsed());
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, ServerConstants::HEADER_CONTENT_TYPE_VALUE)
                        .body(axum::body::Body::from(result))
                        .unwrap());
                }
                Err(e) => {
                    tracing::error!("Failed after {:.2?}: {:?}", start_time.elapsed(), e);
                    return Err(e);
                }
            }
        }
    }
    
    tracing::error!("No image found in request");
    Err(AppError::ImageProcessingError("No image file found".to_string()))
}