use crate::domain::AppError;
use ort::{session::Session, value::Tensor, Error as OrtError};
use image::{ImageBuffer, Rgba, DynamicImage};
use ndarray::Array4;

pub struct ImageProcessor {
    session: Session,
}

impl ImageProcessor {
    const PIXEL_SIZE: u32 = 320;

    pub fn new() -> Result<Self, AppError> {
        // Initialize the ONNX Runtime environment
        ort::init()
            .with_name("rembg")
            .commit()
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        // Create session using the new builder pattern
        let session = Session::builder()
            .map_err(|e| AppError::ModelError(e.to_string()))?
            .commit_from_file("models/silueta.onnx")
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        Ok(Self { session })
    }

    fn calculate_dimensions(&self, orig_width: u32, orig_height: u32) -> (u32, u32) {
        if orig_width > orig_height {
            let height = (orig_height as f32 * (320.0 / orig_width as f32)) as u32;
            (320, height)
        } else {
            let width = (orig_width as f32 * (320.0 / orig_height as f32)) as u32;
            (width, 320)
        }
    }

    fn load_input_image(&self, image_data: &[u8]) -> Result<DynamicImage, AppError> {
        image::load_from_memory(image_data).map_err(|e| AppError::ImageProcessingError(e.to_string()))
    }

    fn store_original_dimensions(&self, img: &DynamicImage) -> (u32, u32) {
        (img.width(), img.height())
    }

    fn resize_image(&self, img: &DynamicImage, resize_width: u32, resize_height: u32) -> DynamicImage {
        img.resize_exact(resize_width, resize_height, image::imageops::FilterType::Lanczos3)
    }

    fn convert_image_to_rgb(&self, img: &DynamicImage) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        img.to_rgb8()
    }

    fn calculate_start_coordinates(&self, resize_width: u32, resize_height: u32) -> (u32, u32) {
        (
            (Self::PIXEL_SIZE - resize_width) / 2,
            (Self::PIXEL_SIZE - resize_height) / 2
        )
    }

    fn create_padded_image_with_black_bg(&self) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        ImageBuffer::from_fn(Self::PIXEL_SIZE, Self::PIXEL_SIZE, |_, _| {
            image::Rgb([0, 0, 0])
        })
    }

    fn copy_resized_image_to_padded_image(&self, padded: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>, rgb_img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>, resize_width: u32, resize_height: u32, start_x: u32, start_y: u32) {
        for y in 0..resize_height {
            for x in 0..resize_width {
                let pixel = rgb_img.get_pixel(x, y);
                padded.put_pixel(start_x + x, start_y + y, *pixel);
            }
        }
    }

    fn prepare_input_tensor_with_normalized_values(&self, padded: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Array4<f32> {
        let shape: [usize; 4] = [1, 3, Self::PIXEL_SIZE as usize, Self::PIXEL_SIZE as usize];
        let mut input_tensor = Array4::zeros(shape);
        for y in 0..Self::PIXEL_SIZE {
            for x in 0..Self::PIXEL_SIZE {
                let pixel = padded.get_pixel(x, y);
                input_tensor[[0, 0, y as usize, x as usize]] = pixel[0] as f32 / 255.0;
                input_tensor[[0, 1, y as usize, x as usize]] = pixel[1] as f32 / 255.0;
                input_tensor[[0, 2, y as usize, x as usize]] = pixel[2] as f32 / 255.0;
            }
        }
        input_tensor
    }


    pub async fn remove_background(&self, image_data: &[u8]) -> Result<Vec<u8>, AppError> {
        // Load the input image
        let img = self.load_input_image(image_data)?;
        
        // Store original dimensions
        let (orig_width, orig_height) = self.store_original_dimensions(&img);
        
        // Calculate dimensions maintaining aspect ratio
        let (resize_width, resize_height) = self.calculate_dimensions(orig_width, orig_height);
        
        // Resize maintaining aspect ratio
        let resized = self.resize_image(&img, resize_width, resize_height);
        let rgb_img = self.convert_image_to_rgb(&resized);
        
        // Create padded image with black background
        let mut padded = self.create_padded_image_with_black_bg();
        let (start_x, start_y) = self.calculate_start_coordinates(resize_width, resize_height);
        
        // Copy resized image to center of padded image
        self.copy_resized_image_to_padded_image(&mut padded, &rgb_img, resize_width, resize_height, start_x, start_y);

        // Prepare input tensor (1, 3, 320, 320) with normalized values
        let input_tensor = self.prepare_input_tensor_with_normalized_values(&padded);

        // Convert to owned array in standard layout and create tensor
        let input_array = input_tensor.as_standard_layout();
        let shape = vec![1i64, 3, 320, 320];
        let data: Vec<f32> = input_array.as_slice().unwrap().to_vec();
        let tensor = Tensor::from_array((shape, data))
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        // Run inference using the new inputs! macro
        let outputs = self.session
            .run(ort::inputs![tensor]?)
            .map_err(|e| AppError::ModelError(e.to_string()))?;

        // Extract output tensor using the new API
        let output_view = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::ModelError(e.to_string()))?;
        
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

impl From<OrtError> for AppError {
    fn from(error: OrtError) -> Self {
        AppError::ModelError(error.to_string())
    }
}