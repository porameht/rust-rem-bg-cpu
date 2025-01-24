pub struct ImageConstants;

impl ImageConstants {
    pub const INFERENCE_PIXEL_SIZE: u32 = 320;
    
    pub const SILUETA_MODEL_PATH: &'static str = "models/silueta.onnx";

    pub const ORT_NAME: &'static str = "rembg";
}

pub struct ServerConstants;

impl ServerConstants {
    pub const DEFAULT_PORT: &'static str = "8000";
    
    pub const MAX_BODY_SIZE: usize = 1024 * 1024 * 10;

    pub const CONTENT_TYPE_PNG: &'static str = "image/png";
    pub const CONTENT_TYPE_JPEG: &'static str = "image/jpeg";
    pub const CONTENT_TYPE_JPG: &'static str = "image/jpg";

    pub const HEADER_CONTENT_TYPE_VALUE: &'static str = "image/png";

    pub const PATH_REMOVE_BACKGROUND: &'static str = "/api/rem-bg";

    pub const FIELD_IMAGE: &'static str = "image";
}