//! Конфигурация

/// Настройки протокола DDP
/// 
/// **Поля:**
/// - `ip`: [String]             - адрес ленты или mdns-имя 
/// - `port`: [Option]<[u16]>    - порт ленты
/// - `timeout`: [Option]<[u8]>  - время ожидания ленты
/// - `mtu`: [Option]<[u16]>     - MTU в сети (для корректного разбиения на пакеты)
pub struct DdpConfig {
    pub ip: String,
    pub port: Option<u16>,
    pub timeout: Option<u8>,
    pub mtu: Option<u16>,
}
