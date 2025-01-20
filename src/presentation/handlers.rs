use axum::{
    extract::{State, Multipart},
    response::{IntoResponse, Response},
    http::{header, StatusCode},
};
use std::sync::Arc;
use crate::application::image_processor::ImageProcessor;
use crate::domain::AppError;

pub async fn remove_background(
    State(processor): State<Arc<ImageProcessor>>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::ImageProcessingError(e.to_string()))? {
        if field.name() == Some("image") {
            let data = field.bytes().await.map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
            let result = processor.remove_background(&data).await?;
            
            // Return the image data with proper headers
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/png")
                .body(axum::body::Body::from(result))
                .unwrap());
        }
    }
    
    Err(AppError::ImageProcessingError("No image file found".to_string()))
} 