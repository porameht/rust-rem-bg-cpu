use image::DynamicImage;
use ndarray::Array4;
use crate::domain::AppError;
use super::constants::preprocessing::*;

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
        
        let resized = img.resize_exact(resize_width, resize_height, image::imageops::FilterType::Triangle);
        let rgb_img = resized.to_rgb8();
        
        let resize_width_usize = resize_width as usize;
        let (start_x, start_y) = self.calculate_start_coordinates(resize_width, resize_height);
        let (start_x_usize, start_y_usize) = (start_x as usize, start_y as usize);
        let pixel_size_usize = self.pixel_size as usize;
        
        let mut input_tensor = Array4::zeros([1, 3, pixel_size_usize, pixel_size_usize]);
        
        let buffer = rgb_img.as_raw();
        
        for (i, pixel) in buffer.chunks_exact(3).enumerate() {
            let x = i % resize_width_usize;
            let y = i / resize_width_usize;
            
            let tensor_x = start_x_usize + x;
            let tensor_y = start_y_usize + y;
            
            let r = unsafe { pixel.get_unchecked(0) };
            let g = unsafe { pixel.get_unchecked(1) };
            let b = unsafe { pixel.get_unchecked(2) };
            
            let normalized_r = (*r as f32) * SCALE_R + OFFSET_R;
            let normalized_g = (*g as f32) * SCALE_G + OFFSET_G;
            let normalized_b = (*b as f32) * SCALE_B + OFFSET_B;
            
            unsafe {
                *input_tensor.uget_mut([0, 0, tensor_y, tensor_x]) = normalized_r;
                *input_tensor.uget_mut([0, 1, tensor_y, tensor_x]) = normalized_g;
                *input_tensor.uget_mut([0, 2, tensor_y, tensor_x]) = normalized_b;
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
