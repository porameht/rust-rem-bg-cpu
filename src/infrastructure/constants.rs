pub struct InfrastructureConstants;

impl InfrastructureConstants {
    pub const DEFAULT_PORT: u16 = 8000;
    pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB
    pub const PATH_REMOVE_BACKGROUND: &'static str = "/api/rem-bg";
    pub const PATH_BATCH_REMOVE_BACKGROUND: &'static str = "/api/batch-rem-bg";
}