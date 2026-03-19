//! Структуры, необходимы для работы serial-порта

use serialport::SerialPort;

/// Информация о serial-порте
/// 
/// **Поля:**
/// - `port`: [Box]<dyn [SerialPort]> - порт устройства (из [serialport])
pub struct SerialDriver {
    pub(crate) port: Box<dyn SerialPort>,
}
