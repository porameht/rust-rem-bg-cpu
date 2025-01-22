use crate::domain::AppError;
use super::{preprocessing::ImagePreprocessor, inference::ModelInference, postprocessing::ImagePostprocessor};

pub struct ImageProcessor {
    preprocessor: ImagePreprocessor,
    inference: ModelInference,
    postprocessor: ImagePostprocessor,
}

impl ImageProcessor {
    const PIXEL_SIZE: u32 = 320;

    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            preprocessor: ImagePreprocessor::new(Self::PIXEL_SIZE),
            inference: ModelInference::new("models/silueta.onnx")?,
            postprocessor: ImagePostprocessor::new(),
        })
    }
    
    pub async fn remove_background(&self, image_data: &[u8]) -> Result<Vec<u8>, AppError> {
        // Load image
        let img = self.preprocessor.load_input_image(image_data)?;

        // Preprocess
        let (input_tensor, orig_dims, resize_dims, start_coords) = 
            self.preprocessor.prepare_for_inference(&img)?;

        // Run inference
        let input_array = input_tensor.as_standard_layout();
        let outputs = self.inference.run(input_array.view())?;

        // Postprocess
        self.postprocessor.process_output(
            outputs,
            &img,
            (orig_dims, resize_dims, start_coords)
        )
    }
}