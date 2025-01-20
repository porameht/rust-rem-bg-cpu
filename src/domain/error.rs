use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

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