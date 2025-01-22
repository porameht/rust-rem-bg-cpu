use ort::{session::{Session, SessionOutputs}, value::Tensor, Error as OrtError};
use ndarray::ArrayView4;
use crate::domain::{AppError, ImageConstants};

pub struct ModelInference {
    session: Session,
}

impl ModelInference {
    pub fn new(model_path: &str) -> Result<Self, AppError> {
        ort::init()
            .with_name(ImageConstants::ORT_NAME)
            .commit()
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        let session = Session::builder()
            .map_err(|e| AppError::ModelError(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        Ok(Self { session })
    }

    pub fn run(&self, input_array: ArrayView4<f32>) -> Result<SessionOutputs, AppError> {
        let tensor = self.create_tensor(input_array)?;
        self.session
            .run(ort::inputs![tensor]?)
            .map_err(|e| AppError::ModelError(e.to_string()))
    }

    fn create_tensor(&self, input_array: ArrayView4<f32>) -> Result<Tensor<f32>, AppError> {
        let shape = vec![1i64, 3, 320, 320];
        let data: Vec<f32> = input_array.as_slice().unwrap().to_vec();
        Tensor::from_array((shape, data))
            .map_err(|e| AppError::ModelError(e.to_string()))
    }
}

impl From<OrtError> for AppError {
    fn from(error: OrtError) -> Self {
        AppError::ModelError(error.to_string())
    }
} 