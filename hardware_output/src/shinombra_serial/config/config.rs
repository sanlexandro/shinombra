//! Конфигурация, необходимая для работы вывода на экран

/// Структура для хранения настроек
///
/// **Поля:**
/// - `port_path`: [String] - путь к устройству
/// - `baud_rate`: [u32]    - скорость обмена данными
#[derive(Default)]
pub struct ShinombraSerialConfig {
    pub port_path: String,
    pub baud_rate: u32,
}
