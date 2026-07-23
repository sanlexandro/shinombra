//! Конфигурация

/// Настройки протокола WledDrgb
/// 
/// **Поля:**
/// - `ip`: [String]             - адрес ленты или mdns-имя 
/// - `port`: [Option]<[u16]>    - порт ленты
/// - `timeout`: [Option]<[u8]>  - время ожидания ленты
/// - `max_fps`: [Option]<[u16]> - максимальный FPS (в зависимости от качества сети)
pub struct WledDrgbConfig {
    pub ip: String,
    pub port: Option<u16>,
    pub timeout: Option<u8>,
    pub max_fps: Option<u16>,
}
