//! Структуры, необходимы для работы serial-порта

use serialport::SerialPort;

/// Информация о serial-порте
///
/// **Поля:**
/// - `port`: [Box]<dyn [SerialPort]> - порт устройства (из [serialport])
/// - `payload`: [Vec]<[u8]>          - буфер (для избежания аллокаций памяти в
///   runtime)
pub struct ShinombraSerial {
    pub(crate) port: Box<dyn SerialPort>,
    pub(crate) payload: Vec<u8>
}
