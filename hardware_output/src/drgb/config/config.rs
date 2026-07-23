//! Конфигурация

pub struct DRGBConfig {
    pub ip: String,
    pub port: Option<u16>,
    pub timeout: Option<u8>,
    pub max_fps: Option<u16>,
}
