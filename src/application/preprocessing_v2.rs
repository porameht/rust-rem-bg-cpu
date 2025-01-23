use image::DynamicImage;
use ndarray::Array4;
use crate::domain::AppError;

pub struct ImagePreprocessorV2 {
    pixel_size: u32,
}

impl ImagePreprocessorV2 {
    pub fn new(pixel_size: u32) -> Self {
        Self { pixel_size }
    }

    pub fn prepare_for_inference(&self, img: &DynamicImage) -> Result<(Array4<f32>, (u32, u32), (u32, u32), (u32, u32)), AppError> {
        let (orig_width, orig_height) = (img.width(), img.height());
        let (resize_width, resize_height) = self.calculate_dimensions(orig_width, orig_height);
        
        let resized = img.resize_exact(resize_width, resize_height, image::imageops::FilterType::Lanczos3);
        let rgb_img = resized.to_rgb8();
        
        // Create tensor with RGB channels (3 channels)
        let mut input_tensor = Array4::zeros([1, 3, self.pixel_size as usize, self.pixel_size as usize]);
        let (start_x, start_y) = self.calculate_start_coordinates(resize_width, resize_height);
        
        // Fill tensor with normalized values
        for y in 0..resize_height {
            for x in 0..resize_width {
                let tensor_x = (start_x + x) as usize;
                let tensor_y = (start_y + y) as usize;
                
                let pixel = rgb_img.get_pixel(x, y);
                
                // Normalize using same values as Python u2net (pixel_value / 255.0 - mean) / std 
                // reduce effect of different brightness in image
                // Red (R): (x - 0.485) / 0.229
                // Green (G): (x - 0.456) / 0.224
                // Blue (B): (x - 0.406) / 0.225
                input_tensor[[0, 0, tensor_y, tensor_x]] = (pixel[0] as f32 / 255.0 - 0.485) / 0.229;
                input_tensor[[0, 1, tensor_y, tensor_x]] = (pixel[1] as f32 / 255.0 - 0.456) / 0.224;
                input_tensor[[0, 2, tensor_y, tensor_x]] = (pixel[2] as f32 / 255.0 - 0.406) / 0.225;
            }
        }

        Ok((input_tensor, (orig_width, orig_height), (resize_width, resize_height), (start_x, start_y)))
    }

    fn calculate_dimensions(&self, orig_width: u32, orig_height: u32) -> (u32, u32) {
        if orig_width > orig_height {
            let height = (orig_height as f32 * (self.pixel_size as f32 / orig_width as f32)) as u32;
            (self.pixel_size, height)
        } else {
            let width = (orig_width as f32 * (self.pixel_size as f32 / orig_height as f32)) as u32;
            (width, self.pixel_size)
        }
    }

    fn calculate_start_coordinates(&self, width: u32, height: u32) -> (u32, u32) {
        ((self.pixel_size - width) / 2, (self.pixel_size - height) / 2)
    }
} 