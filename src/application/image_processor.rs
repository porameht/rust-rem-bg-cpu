use crate::domain::AppError;
use ort::{Environment, Session, SessionBuilder, Value};
use image::{ImageBuffer, Rgba};
use ndarray::{Array4, ArrayView4};
use std::sync::Arc;

pub struct ImageProcessor {
    session: Session,
    _environment: Arc<Environment>,
}

impl ImageProcessor {
    pub fn new() -> Result<Self, AppError> {
        let environment = Arc::new(Environment::builder()
            .with_name("rembg")
            .build()
            .map_err(|e| AppError::ModelError(e.to_string()))?);

        let session = SessionBuilder::new(&environment)
            .map_err(|e| AppError::ModelError(e.to_string()))?
            .with_model_from_file("models/u2net.onnx")
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        Ok(Self { 
            session,
            _environment: environment,
        })
    }

    pub async fn remove_background(&self, image_data: &[u8]) -> Result<Vec<u8>, AppError> {
        // Load the input image
        let img = image::load_from_memory(image_data)
            .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
        
        // Store original dimensions
        let (orig_width, orig_height) = (img.width(), img.height());
        
        // Calculate dimensions maintaining aspect ratio
        let (resize_width, resize_height) = if orig_width > orig_height {
            let height = (orig_height as f32 * (320.0 / orig_width as f32)) as u32;
            (320, height)
        } else {
            let width = (orig_width as f32 * (320.0 / orig_height as f32)) as u32;
            (width, 320)
        };
        
        // Resize maintaining aspect ratio
        let resized = img.resize_exact(resize_width, resize_height, image::imageops::FilterType::Lanczos3);
        let rgb_img = resized.to_rgb8();
        
        // Create padded image with black background
        let mut padded = ImageBuffer::new(320, 320);
        let start_x = (320 - resize_width) / 2;
        let start_y = (320 - resize_height) / 2;
        
        // Copy resized image to center of padded image
        for y in 0..resize_height {
            for x in 0..resize_width {
                let pixel = rgb_img.get_pixel(x, y);
                padded.put_pixel(start_x + x, start_y + y, *pixel);
            }
        }

        // Prepare input tensor (1, 3, 320, 320) with normalized values
        let mut input_tensor = Array4::zeros((1, 3, 320, 320));
        for y in 0..320u32 {
            for x in 0..320u32 {
                let pixel = padded.get_pixel(x, y);
                input_tensor[[0, 0, y as usize, x as usize]] = pixel[0] as f32 / 255.0;
                input_tensor[[0, 1, y as usize, x as usize]] = pixel[1] as f32 / 255.0;
                input_tensor[[0, 2, y as usize, x as usize]] = pixel[2] as f32 / 255.0;
            }
        }

        // Create input Value for the model
        let input_tensor = input_tensor.into_dyn();
        let standard_tensor = input_tensor.as_standard_layout();
        let input_values = vec![Value::from_array(self.session.allocator(), &standard_tensor)
            .map_err(|e| AppError::ModelError(e.to_string()))?];

        // Run inference
        let outputs = self.session
            .run(input_values)
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        // Process the output mask
        let output_tensor = outputs[0]
            .try_extract::<f32>()
            .map_err(|e| AppError::ModelError(e.to_string()))?;
        
        let output_view = output_tensor.view();
        let output_shape = output_view.shape();
        
        // Create output image with transparency, using original dimensions
        let mut output_img = ImageBuffer::new(orig_width, orig_height);
        
        // Scale the mask back to original dimensions
        let scaled_img = img.to_rgba8();
        let scaled_mask = ImageBuffer::from_fn(orig_width, orig_height, |x, y| {
            let x_ratio = x as f32 / orig_width as f32;
            let y_ratio = y as f32 / orig_height as f32;
            
            // Map back to padded coordinates
            let mask_x = ((x_ratio * resize_width as f32) + start_x as f32) as usize;
            let mask_y = ((y_ratio * resize_height as f32) + start_y as f32) as usize;
            
            let mask_value = output_view[[0, 0, mask_y.min(319), mask_x.min(319)]].max(0.0).min(1.0);
            image::Luma([(mask_value * 255.0) as u8])
        });

        // Apply the scaled mask
        for y in 0..orig_height {
            for x in 0..orig_width {
                let original_pixel = scaled_img.get_pixel(x, y);
                let mask_value = scaled_mask.get_pixel(x, y)[0];
                output_img.put_pixel(x, y, Rgba([
                    original_pixel[0],
                    original_pixel[1],
                    original_pixel[2],
                    mask_value,
                ]));
            }
        }

        // Encode the result as PNG
        let mut output_buffer = Vec::new();
        image::DynamicImage::ImageRgba8(output_img)
            .write_to(&mut std::io::Cursor::new(&mut output_buffer), image::ImageOutputFormat::Png)
            .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;

        Ok(output_buffer)
    }
} 