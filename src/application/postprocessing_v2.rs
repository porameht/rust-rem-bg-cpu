use image::{DynamicImage, ImageEncoder};
use rayon::prelude::*;
use crate::domain::AppError;
use crate::application::constants::{postprocessing::*, edge_detection::*};

pub struct ImagePostprocessorV2 {
    pixel_size: u32,
}

impl ImagePostprocessorV2 {
    pub fn new(pixel_size: u32) -> Self {
        Self { pixel_size }
    }

    pub fn process_output(
        &self,
        outputs: Vec<f32>,
        img: &DynamicImage,
        dimensions: ((u32, u32), (u32, u32), (u32, u32)),
    ) -> Result<Vec<u8>, AppError> {
        let ((orig_width, orig_height), (resize_width, resize_height), (start_x, start_y)) = dimensions;
        let pixel_size_usize = self.pixel_size as usize;

        let mut output_buffer = Vec::with_capacity((orig_width * orig_height * 4) as usize);
        let encoder = image::codecs::png::PngEncoder::new(&mut output_buffer);

        let img_rgba = img.to_rgba8();
        let rgba_buffer = img_rgba.as_raw();

        let orig_width_usize = orig_width as usize;
        let total_pixels = orig_width_usize * orig_height as usize;
        let resize_width_f32 = resize_width as f32;
        let resize_height_f32 = resize_height as f32;
        let x_scale = 1.0 / orig_width as f32;
        let y_scale = 1.0 / orig_height as f32;

        let mut rgba_data = vec![0u8; total_pixels * 4];
        let outputs_slice = outputs.as_slice();

        // First pass: Calculate raw alpha values with bilinear interpolation
        let mut alpha_buffer = vec![0f32; total_pixels];
        alpha_buffer.par_iter_mut().enumerate().for_each(|(i, alpha)| {
            let x = i % orig_width_usize;
            let y = i / orig_width_usize;

            // Calculate exact mask position with floating point precision
            let mask_x = (x as f32 * x_scale * resize_width_f32) + start_x as f32;
            let mask_y = (y as f32 * y_scale * resize_height_f32) + start_y as f32;

            // Bilinear interpolation
            let x0 = mask_x.floor() as usize;
            let y0 = mask_y.floor() as usize;
            let x1 = (x0 + 1).min(pixel_size_usize - 1);
            let y1 = (y0 + 1).min(pixel_size_usize - 1);
            
            let dx = mask_x - x0 as f32;
            let dy = mask_y - y0 as f32;

            let get_value = |x, y| outputs_slice.get(y * pixel_size_usize + x).copied().unwrap_or(0.0);
            
            let v00 = get_value(x0, y0);
            let v01 = get_value(x0, y1);
            let v10 = get_value(x1, y0);
            let v11 = get_value(x1, y1);
            
            let interpolated = 
                v00 * (1.0 - dx) * (1.0 - dy) +
                v10 * dx * (1.0 - dy) +
                v01 * (1.0 - dx) * dy +
                v11 * dx * dy;

            *alpha = interpolated.clamp(0.0, 1.0);
        });

        // Second pass: Edge detection and alpha refinement
        rgba_data.par_chunks_exact_mut(4).enumerate().for_each(|(i, chunk)| {
            let x = i % orig_width_usize;
            let y = i / orig_width_usize;
            
            // Edge detection using Laplace operator
            let edge_score = self.calculate_edge_score(x, y, &alpha_buffer, orig_width_usize, orig_height as usize);
            
            let alpha = alpha_buffer[i];
            let smoothed_alpha = if edge_score > EDGE_DETECTION_THRESHOLD {
                // Use cubic interpolation for edges
                let t = ((alpha - EDGE_ALPHA_MIN) / EDGE_ALPHA_RANGE).clamp(0.0, 1.0);
                (t * t * (3.0 - 2.0 * t)) * EDGE_BLEND_FACTOR + alpha * (1.0 - EDGE_BLEND_FACTOR)
            } else {
                // Smootherstep for non-edge areas
                let t = ((alpha - SMOOTH_ALPHA_MIN) / SMOOTH_ALPHA_RANGE).clamp(0.0, 1.0);
                t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
            };

            let pixel_start = i * 4;
            chunk[0] = rgba_buffer[pixel_start];
            chunk[1] = rgba_buffer[pixel_start + 1];
            chunk[2] = rgba_buffer[pixel_start + 2];
            chunk[3] = (smoothed_alpha * 255.0).round() as u8;
        });

        encoder.write_image(
            &rgba_data,
            orig_width,
            orig_height,
            image::ColorType::Rgba8,
        ).map_err(|e| AppError::ImageProcessingError(e.to_string()))?;

        Ok(output_buffer)
    }

    fn calculate_edge_score(&self, x: usize, y: usize, alpha: &[f32], width: usize, height: usize) -> f32 {
        let get_alpha = |dx: i32, dy: i32| {
            let nx = (x as i32 + dx).clamp(0, width as i32 - 1) as usize;
            let ny = (y as i32 + dy).clamp(0, height as i32 - 1) as usize;
            alpha[ny * width + nx]
        };

        // Laplace edge detection kernel
        let score = 
            KERNEL_SURROUNDING * get_alpha(-1, -1) + KERNEL_SURROUNDING * get_alpha(0, -1) + KERNEL_SURROUNDING * get_alpha(1, -1) +
            KERNEL_SURROUNDING * get_alpha(-1, 0) + KERNEL_CENTER * get_alpha(0, 0) + KERNEL_SURROUNDING * get_alpha(1, 0) +
            KERNEL_SURROUNDING * get_alpha(-1, 1) + KERNEL_SURROUNDING * get_alpha(0, 1) + KERNEL_SURROUNDING * get_alpha(1, 1);

        score.abs().min(1.0)
    }
}
