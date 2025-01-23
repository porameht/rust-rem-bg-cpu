use crate::domain::{AppError, constants::ImageConstants};
use super::{preprocessing::ImagePreprocessor, inference::ModelInference, postprocessing::ImagePostprocessor};

pub struct ImageProcessor {
    preprocessor: ImagePreprocessor,
    inference: ModelInference,
    postprocessor: ImagePostprocessor,
}

impl ImageProcessor {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            preprocessor: ImagePreprocessor::new(ImageConstants::INFERENCE_PIXEL_SIZE),
            inference: ModelInference::new(ImageConstants::SILUETA_MODEL_PATH)?,
            postprocessor: ImagePostprocessor::new(),
        })
    }
    
    pub async fn remove_background(&self, image_data: &[u8]) -> Result<Vec<u8>, AppError> {
        let img = self.preprocessor.load_input_image(image_data)?;

        let (input_tensor, orig_dims, resize_dims, start_coords) = 
            self.preprocessor.prepare_for_inference(&img)?;

        let input_array = input_tensor.as_standard_layout();
        let outputs = self.inference.run(input_array.view())?;

        self.postprocessor.process_output(
            outputs,
            &img,
            (orig_dims, resize_dims, start_coords)
        )
    }
}