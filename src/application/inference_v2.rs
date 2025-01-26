use ort::{session::Session, value::Tensor};
use ndarray::ArrayView4;
use crate::domain::AppError;
use crate::application::constants::inference::*;

pub struct ModelInferenceV2 {
    session: Session,
}

impl ModelInferenceV2 {
    pub fn new(model_path: &str) -> Result<Self, AppError> {
        ort::init()
            .with_name(ORT_NAME)
            .commit()
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        let session = Session::builder()
            .map_err(|e| AppError::ModelError(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        Ok(Self { session })
    }

    pub fn run(&self, input: ArrayView4<f32>) -> Result<Vec<f32>, AppError> {
        let tensor = self.create_tensor(input)?;
        let outputs = self.session
            .run(ort::inputs![tensor]?)
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        let output_tensor = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        Ok(output_tensor.view().iter().copied().collect())
    }

    fn create_tensor(&self, input_array: ArrayView4<f32>) -> Result<Tensor<f32>, AppError> {
        let shape = vec![1i64, 3, 320, 320]; // Changed back to 3 channels to match model
        let data: Vec<f32> = input_array.as_slice().unwrap().to_vec();
        Tensor::from_array((shape, data))
            .map_err(|e| AppError::ModelError(e.to_string()))
    }
}