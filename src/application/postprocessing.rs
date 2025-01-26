// use image::{ImageBuffer, Rgba, DynamicImage, Luma};
// use crate::domain::AppError;
// use ort::session::SessionOutputs;

// pub struct ImagePostprocessor;

// impl ImagePostprocessor {
//     pub fn new() -> Self {
//         Self
//     }

//     pub fn process_output(
//         &self,
//         outputs: SessionOutputs,
//         img: &DynamicImage,
//         dimensions: ((u32, u32), (u32, u32), (u32, u32))
//     ) -> Result<Vec<u8>, AppError> {
//         let ((orig_width, orig_height), (resize_width, resize_height), (start_x, start_y)) = dimensions;
        
//         let output_tensor = outputs[0]
//             .try_extract_tensor::<f32>()
//             .map_err(|e| AppError::ModelError(e.to_string()))?;
        
//         let output_view = output_tensor.view();

//         let mut output_img = ImageBuffer::new(orig_width, orig_height);
        
//         // Scale the mask back to original dimensions
//         let scaled_img = img.to_rgba8();
//         let scaled_mask = ImageBuffer::from_fn(orig_width, orig_height, |x, y| {
//             let x_ratio = x as f32 / orig_width as f32;
//             let y_ratio = y as f32 / orig_height as f32;
            
//             // Map back to padded coordinates
//             let mask_x = ((x_ratio * resize_width as f32) + start_x as f32) as usize;
//             let mask_y = ((y_ratio * resize_height as f32) + start_y as f32) as usize;
            
//             let mask_value = output_view[[0, 0, mask_y.min(319), mask_x.min(319)]].max(0.0).min(1.0);
//             image::Luma([(mask_value * 255.0) as u8])
//         });

//         self.apply_mask(
//             &scaled_img,
//             &scaled_mask,
//             &mut output_img,
//             (orig_width, orig_height)
//         );

//         self.encode_result(&output_img)
//     }

//     fn apply_mask(
//         &self,
//         scaled_img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
//         scaled_mask: &ImageBuffer<Luma<u8>, Vec<u8>>,
//         output_img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
//         (width, height): (u32, u32)
//     ) {
//         for y in 0..height {
//             for x in 0..width {
//                 let original_pixel = scaled_img.get_pixel(x, y);
//                 let mask_value = scaled_mask.get_pixel(x, y)[0];
//                 output_img.put_pixel(x, y, Rgba([
//                     original_pixel[0],
//                     original_pixel[1],
//                     original_pixel[2],
//                     mask_value,
//                 ]));
//             }
//         }
//     }

//     fn encode_result(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<u8>, AppError> {
//         let mut output_buffer = Vec::new();
//         DynamicImage::ImageRgba8(image.clone())
//             .write_to(&mut std::io::Cursor::new(&mut output_buffer), image::ImageOutputFormat::Png)
//             .map_err(|e| AppError::ImageProcessingError(e.to_string()))?;
//         Ok(output_buffer)
//     }
// } 