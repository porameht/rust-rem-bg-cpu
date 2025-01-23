use crate::domain::{AppError, constants::ImageConstants};
use super::{preprocessing_v2::ImagePreprocessorV2, inference_v2::ModelInferenceV2, postprocessing_v2::ImagePostprocessorV2};

pub struct ImageProcessor {
    preprocessor: ImagePreprocessorV2,
    inference: ModelInferenceV2,
    postprocessor: ImagePostprocessorV2,
}

impl ImageProcessor {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            preprocessor: ImagePreprocessorV2::new(ImageConstants::INFERENCE_PIXEL_SIZE),
            inference: ModelInferenceV2::new(ImageConstants::SILUETA_MODEL_PATH)?,
            postprocessor: ImagePostprocessorV2::new(ImageConstants::INFERENCE_PIXEL_SIZE),
        })
    }
    
    pub async fn remove_background(&self, image_data: &[u8]) -> Result<Vec<u8>, AppError> {
        let img = image::load_from_memory(image_data)
            .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;

        let (input_tensor, orig_dims, resize_dims, start_coords) = 
            self.preprocessor.prepare_for_inference(&img)?;

        let outputs = self.inference.run(input_tensor.view())?;

        self.postprocessor.process_output(
            outputs,
            &img,
            (orig_dims, resize_dims, start_coords)
        )
    }
}