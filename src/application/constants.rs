//! Constants used across the application layer

/// Image preprocessing constants
pub mod preprocessing {
    /// Normalization constants for RGB channels
    pub const SCALE_R: f32 = 1.0 / (255.0 * 0.229);
    pub const OFFSET_R: f32 = -0.485 / 0.229;
    pub const SCALE_G: f32 = 1.0 / (255.0 * 0.224);
    pub const OFFSET_G: f32 = -0.456 / 0.224;
    pub const SCALE_B: f32 = 1.0 / (255.0 * 0.225);
    pub const OFFSET_B: f32 = -0.406 / 0.225;
}

/// Image postprocessing constants
pub mod postprocessing {
    /// Edge detection threshold
    pub const EDGE_DETECTION_THRESHOLD: f32 = 0.1;
    
    /// Alpha refinement constants for edge areas
    pub const EDGE_ALPHA_MIN: f32 = 0.2;
    pub const EDGE_ALPHA_RANGE: f32 = 0.6;
    pub const EDGE_BLEND_FACTOR: f32 = 0.8;
    
    /// Alpha refinement constants for non-edge areas
    pub const SMOOTH_ALPHA_MIN: f32 = 0.1;
    pub const SMOOTH_ALPHA_RANGE: f32 = 0.8;
}

/// Laplace edge detection kernel constants
pub mod edge_detection {
    pub const KERNEL_CENTER: f32 = 8.0;
    pub const KERNEL_SURROUNDING: f32 = -1.0;
}

pub mod inference {
    pub const ORT_NAME: &'static str = "rembg";
}

pub mod image_processor {
    pub const INFERENCE_PIXEL_SIZE: u32 = 320;
    
    pub const SILUETA_MODEL_PATH: &'static str = "models/silueta.onnx";
}