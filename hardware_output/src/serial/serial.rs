//! Реализация связи с железом по Serial

use crate::{
    serial::{config::SerialDriverConfig, types::SerialDriver},
    HardwareEvents,
};
use algorithms::color::types::RGBPixel;
use serialport;
use std::time::Duration;

/// Реализация методов SerialDriver
impl SerialDriver {
    /// Конструктор
    ///
    /// **Аргументы:**
    /// - `port_path`: &[str] - путь к устройству (например, "/dev/ttyUSB0")
    /// - `baud_rate`: [u32]  - скорость обмена данными
    pub fn new(config: SerialDriverConfig) -> Self {
        // Настройка порта
        let port = serialport::new(config.port_path, config.baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
            .expect("Failed to open port");

        // Даем время на перезагрузку после открытия порта
        std::thread::sleep(std::time::Duration::from_secs(2));

        return Self { port };
    }

    /// Отправка массива цвета на устройство
    ///
    /// **Поля:**
    /// - `colors`: &[[RGBPixel]] - массив из RGBPixel
    pub fn internal_send(&mut self, colors: &[RGBPixel]) -> Result<(), HardwareEvents> {
        // Формируем пакет: [Префикс] + [RGB данные]
        let mut payload = Vec::with_capacity(2 + colors.len() * 3);
        payload.extend_from_slice(b"AD"); // Magic Word (AmbiData)

        // Сохраняем для отправки
        for color in colors {
            payload.push(color.red);
            payload.push(color.green);
            payload.push(color.blue);
        }

        // Отправляем всё одним махом
        match self.port.write_all(&payload) {
            Ok(_) => Ok(()),
            Err(error) => {
                let event = match error.kind() {
                    // Устройство физически отключено или порт закрыт системой
                    std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::AddrNotAvailable
                    | std::io::ErrorKind::NotFound => HardwareEvents::Disconnected,

                    // Ошибки прав доступа
                    std::io::ErrorKind::PermissionDenied => HardwareEvents::NoAccess,

                    // Временные сбои: прерывание системным вызовом или таймаут
                    // Тут имеет смысл попробовать отправить еще раз
                    std::io::ErrorKind::Interrupted | std::io::ErrorKind::TimedOut => {
                        HardwareEvents::RetryNeeded
                    }

                    // Всё остальное, что мы не ожидали
                    _ => {
                        HardwareEvents::InternalError(error.to_string())
                    }
                };
                Err(event)
            }
        }
    }
}

impl Drop for SerialDriver {
    fn drop(&mut self) {
        // Порт закроется автоматически
    }
}
