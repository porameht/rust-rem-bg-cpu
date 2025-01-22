use image::{DynamicImage, ImageBuffer, Rgb};
use ndarray::Array4;
use crate::domain::AppError;

pub struct ImagePreprocessor {
    pixel_size: u32,
}

impl ImagePreprocessor {
    pub fn new(pixel_size: u32) -> Self {
        Self { pixel_size }
    }

    pub fn load_input_image(&self, image_data: &[u8]) -> Result<DynamicImage, AppError> {
        image::load_from_memory(image_data)
            .map_err(|e| AppError::ImageProcessingError(e.to_string()))
    }

    pub fn prepare_for_inference(&self, img: &DynamicImage) -> Result<(Array4<f32>, (u32, u32), (u32, u32), (u32, u32)), AppError> {
        let (orig_width, orig_height) = (img.width(), img.height());
        let (resize_width, resize_height) = self.calculate_dimensions(orig_width, orig_height);
        
        let resized = self.resize_image(img, resize_width, resize_height);
        let rgb_img = resized.to_rgb8();
        
        let mut padded = self.create_padded_image();
        let (start_x, start_y) = self.calculate_start_coordinates(resize_width, resize_height);
        
        self.copy_to_padded_image(&mut padded, &rgb_img, resize_width, resize_height, start_x, start_y);
        let input_tensor = self.prepare_normalized_tensor(&padded);

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

    fn resize_image(&self, img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
        img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
    }

    fn create_padded_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        ImageBuffer::from_fn(self.pixel_size, self.pixel_size, |_, _| {
            Rgb([0, 0, 0])
        })
    }

    fn calculate_start_coordinates(&self, width: u32, height: u32) -> (u32, u32) {
        (
            (self.pixel_size - width) / 2,
            (self.pixel_size - height) / 2
        )
    }

    fn copy_to_padded_image(
        &self,
        padded: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
        rgb_img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
    ) {
        for y in 0..height {
            for x in 0..width {
                let pixel = rgb_img.get_pixel(x, y);
                padded.put_pixel(start_x + x, start_y + y, *pixel);
            }
        }
    }

    fn prepare_normalized_tensor(&self, padded: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Array4<f32> {
        let shape = [1, 3, self.pixel_size as usize, self.pixel_size as usize];
        let mut input_tensor = Array4::zeros(shape);
        
        for y in 0..self.pixel_size {
            for x in 0..self.pixel_size {
                let pixel = padded.get_pixel(x, y);
                input_tensor[[0, 0, y as usize, x as usize]] = pixel[0] as f32 / 255.0;
                input_tensor[[0, 1, y as usize, x as usize]] = pixel[1] as f32 / 255.0;
                input_tensor[[0, 2, y as usize, x as usize]] = pixel[2] as f32 / 255.0;
            }
        }
        
        input_tensor
    }
} 