//! Реализация методов для [HardwareEvents]

use crate::HardwareEvents;
use std::io;

/// Превращение стандартных io ошибок в enum [HardwareEvents]
impl From<io::Error> for HardwareEvents {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            // Устройство физически отключилось
            std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::NotConnected
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::AddrNotAvailable
            | std::io::ErrorKind::NotFound => HardwareEvents::Disconnected,

            // Проблемы с правами: нет прав root/admin, порт заблокирован
            // или устройство занято другим процессом.
            std::io::ErrorKind::PermissionDenied => HardwareEvents::NoAccess,

            // Временные сбои и задержки, при которых повторный запрос имеет
            // высокий шанс на успех.
            std::io::ErrorKind::Interrupted
            | std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::WouldBlock        // Порт в неблокирующем режиме временно не готов
            | std::io::ErrorKind::ResourceBusy      // Шина/устройство временно занято
            | std::io::ErrorKind::ConnectionRefused // Хост поднялся, но порт ещё "разогревается"
            | std::io::ErrorKind::AddrInUse => HardwareEvents::RetryNeeded,

            // Невалидные данные, битые буферы, переполнения или неизведанные ошибки ОС.
            // Передаем исходный текст ошибки.
            _ => HardwareEvents::InternalError(error.to_string()),
        }
    }
}
