use image::{DynamicImage, ImageEncoder};
use crate::domain::AppError;

pub struct ImagePostprocessorV2;

impl ImagePostprocessorV2 {
    pub fn new() -> Self {
        Self
    }

    pub fn process_output(
        &self,
        outputs: Vec<f32>,
        img: &DynamicImage,
        dimensions: ((u32, u32), (u32, u32), (u32, u32))
    ) -> Result<Vec<u8>, AppError> {
        let ((orig_width, orig_height), (resize_width, resize_height), (start_x, start_y)) = dimensions;
        
        let mut output_buffer = Vec::with_capacity((orig_width * orig_height * 4) as usize);
        let encoder = image::codecs::png::PngEncoder::new(&mut output_buffer);
        
        let rgba_data: Vec<u8> = {
            let mut data = Vec::with_capacity((orig_width * orig_height * 4) as usize);
            let img_rgba = img.to_rgba8();
            
            for y in 0..orig_height {
                for x in 0..orig_width {
                    let x_ratio = x as f32 / orig_width as f32;
                    let y_ratio = y as f32 / orig_height as f32;
                    
                    let mask_x = ((x_ratio * resize_width as f32) + start_x as f32) as usize;
                    let mask_y = ((y_ratio * resize_height as f32) + start_y as f32) as usize;
                    
                    // Get mask value from first output
                    let mask_idx = mask_y * 320 + mask_x;
                    let mask_value = outputs[mask_idx].max(0.0).min(1.0);
                    
                    let pixel = img_rgba.get_pixel(x, y);
                    
                    // Apply trimap-like thresholding for better edge detection
                    let alpha = if mask_value > 0.9 {
                        255
                    } else if mask_value < 0.1 {
                        0
                    } else {
                        (mask_value * 255.0) as u8
                    };
                    
                    data.extend_from_slice(&[
                        pixel[0],
                        pixel[1],
                        pixel[2],
                        alpha,
                    ]);
                }
            }
            data
        };

        encoder.write_image(
            &rgba_data,
            orig_width,
            orig_height,
            image::ColorType::Rgba8
        ).map_err(|e| AppError::ImageProcessingError(e.to_string()))?;

        Ok(output_buffer)
    }
} 