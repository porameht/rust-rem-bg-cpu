use image::{DynamicImage, ImageEncoder};
use rayon::prelude::*;
use crate::domain::AppError;

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

        // Precompute values for efficiency
        let orig_width_usize = orig_width as usize;
        let total_pixels = orig_width_usize * orig_height as usize;
        let resize_width_f32 = resize_width as f32;
        let resize_height_f32 = resize_height as f32;
        let start_x_usize = start_x as usize;
        let start_y_usize = start_y as usize;
        let x_scale = 1.0 / orig_width as f32;
        let y_scale = 1.0 / orig_height as f32;

        // Preallocate and parallel process the output buffer
        let mut rgba_data = vec![0u8; total_pixels * 4];
        let outputs_slice = outputs.as_slice();

        rgba_data
            .par_chunks_exact_mut(4)
            .enumerate()
            .for_each(|(i, chunk)| {
                let x = i % orig_width_usize;
                let y = i / orig_width_usize;

                // Calculate mask position with floor for safety
                let mask_x = (x as f32 * x_scale * resize_width_f32).floor() as usize + start_x_usize;
                let mask_y = (y as f32 * y_scale * resize_height_f32).floor() as usize + start_y_usize;

                // Get mask value with bounds checking
                let mask_value = outputs_slice
                    .get(mask_y * pixel_size_usize + mask_x)
                    .copied()
                    .unwrap_or(0.0)
                    .clamp(0.0, 1.0);

                // Direct buffer access for original pixel data
                let pixel_start = i * 4;
                chunk[0] = rgba_buffer[pixel_start];
                chunk[1] = rgba_buffer[pixel_start + 1];
                chunk[2] = rgba_buffer[pixel_start + 2];

                // Optimized alpha calculation using arithmetic
                chunk[3] = match mask_value {
                    v if v > 0.9 => 255,
                    v if v < 0.1 => 0,
                    v => (v * 255.0) as u8,
                };
            });

        encoder.write_image(
            &rgba_data,
            orig_width,
            orig_height,
            image::ColorType::Rgba8,
        ).map_err(|e| AppError::ImageProcessingError(e.to_string()))?;

        Ok(output_buffer)
    }
}
