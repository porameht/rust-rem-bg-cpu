pub struct PresentationConstants;

impl PresentationConstants {
    // Field names for multipart form data
    pub const FIELD_IMAGE: &'static str = "image";
    pub const FIELD_IMAGES: &'static str = "images";

    // Content types
    pub const CONTENT_TYPE_PNG: &'static str = "image/png";
    pub const CONTENT_TYPE_JPEG: &'static str = "image/jpeg";
    pub const CONTENT_TYPE_JPG: &'static str = "image/jpg";

    // Response headers
    pub const HEADER_CONTENT_TYPE_VALUE: &'static str = "image/png";
    pub const HEADER_CONTENT_TYPE_ZIP: &'static str = "application/zip";

    // Error messages
    pub const ERROR_UNSUPPORTED_IMAGE_FORMAT: &'static str = "Unsupported image format. Only PNG and JPEG/JPG are supported";
    pub const ERROR_NO_IMAGE_FOUND: &'static str = "No image file found";
    pub const ERROR_NO_IMAGES_PROCESSED: &'static str = "No images were successfully processed";
    pub const ERROR_ZIP_CREATE: &'static str = "Failed to create zip file";
    pub const ERROR_ZIP_WRITE: &'static str = "Failed to write to zip file";
    pub const ERROR_ZIP_FINALIZE: &'static str = "Failed to finalize zip file";
}