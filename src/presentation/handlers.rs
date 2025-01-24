use axum::{
    extract::{State, Multipart},
    response::Response,
    http::{header, StatusCode},
};
use std::sync::Arc;
use crate::application::image_processor::ImageProcessor;
use crate::domain::{AppError, ServerConstants};
use tracing;
use tokio::task;
use uuid::Uuid;
use std::io::Write;

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
            if let Some(content_type) = field.content_type() {
                if content_type != ServerConstants::CONTENT_TYPE_PNG && 
                   content_type != ServerConstants::CONTENT_TYPE_JPEG && 
                   content_type != ServerConstants::CONTENT_TYPE_JPG {
                    tracing::error!("Unsupported image format: {}", content_type);
                    return Err(AppError::ImageProcessingError(
                        "Unsupported image format. Only PNG and JPEG/JPG are supported".to_string()
                    ));
                }
            }

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

pub async fn batch_remove_background(
    State(processor): State<Arc<ImageProcessor>>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let start_time = std::time::Instant::now();
    tracing::info!("Processing batch background removal request");
    
    let mut processed_images = Vec::new();
    let mut tasks = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to process multipart form: {}", e);
        AppError::ImageProcessingError(e.to_string())
    })? {
        if field.name() == Some(ServerConstants::FIELD_IMAGES) {
            if let Some(content_type) = field.content_type() {
                if content_type != ServerConstants::CONTENT_TYPE_PNG && 
                   content_type != ServerConstants::CONTENT_TYPE_JPEG && 
                   content_type != ServerConstants::CONTENT_TYPE_JPG {
                    tracing::error!("Unsupported image format: {}", content_type);
                    continue;
                }
            }

            let data = field.bytes().await.map_err(|e| {
                tracing::error!("Failed to read image data: {}", e);
                AppError::ImageProcessingError(e.to_string())
            })?;

            let processor = Arc::clone(&processor);
            let task = task::spawn(async move {
                let result = processor.remove_background(&data).await?;
                Ok::<_, AppError>(result)
            });
            tasks.push(task);
        }
    }

    for task in tasks {
        match task.await {
            Ok(Ok(result)) => processed_images.push(result),
            Ok(Err(e)) => {
                tracing::error!("Failed to process image: {:?}", e);
                continue;
            }
            Err(e) => {
                tracing::error!("Task join error: {:?}", e);
                continue;
            }
        }
    }

    if processed_images.is_empty() {
        tracing::error!("No images were successfully processed");
        return Err(AppError::ImageProcessingError("No images were successfully processed".to_string()));
    }

    // Create a zip file containing all processed images
    let mut zip_buffer = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        for (index, image_data) in processed_images.iter().enumerate() {
            let filename = format!("processed_image_{}.png", index + 1);
            zip.start_file(&filename, options).map_err(|e| {
                AppError::ImageProcessingError(format!("Failed to create zip file: {}", e))
            })?;
            zip.write_all(image_data).map_err(|e| {
                AppError::ImageProcessingError(format!("Failed to write to zip file: {}", e))
            })?;
        }
        zip.finish().map_err(|e| {
            AppError::ImageProcessingError(format!("Failed to finalize zip file: {}", e))
        })?;
    }

    tracing::info!("Batch processing completed - took {:.2?}", start_time.elapsed());
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"processed_images_{}.zip\"", Uuid::new_v4())
        )
        .body(axum::body::Body::from(zip_buffer))
        .unwrap())
}