use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use ort::Error as OrtError;

#[derive(Debug)]
pub enum AppError {
    ImageProcessingError(String),
    ModelError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ImageProcessingError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ModelError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
} 

pub struct ErrorMessages;

impl ErrorMessages {
    pub const FAILED_TO_INITIALIZE_IMAGE_PROCESSOR: &'static str = "Failed to initialize image processor";
}

impl From<OrtError> for AppError {
    fn from(error: OrtError) -> Self {
        AppError::ModelError(error.to_string())
    }
} 