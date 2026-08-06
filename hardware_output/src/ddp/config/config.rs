//! Конфигурация

/// Настройки протокола DDP
/// 
/// **Поля:**
/// - `ip`: [String]             - адрес ленты или mdns-имя 
/// - `port`: [Option]<[u16]>    - порт ленты
/// - `mtu`: [Option]<[u16]>     - MTU в сети (для корректного разбиения на пакеты)
pub struct DdpConfig {
    pub ip: String,
    pub port: Option<u16>,
    pub mtu: Option<u16>,
}
