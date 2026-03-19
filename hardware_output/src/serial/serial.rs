//! Реализация связи с железом по Serial

use crate::serial::types::SerialDriver;
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
    pub fn new(port_path: &str, baud_rate: u32) -> Self {
        // Настройка порта
        let port = serialport::new(port_path, baud_rate)
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
    pub fn internal_send(&mut self, colors: &[RGBPixel]) {
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
        self.port.write_all(&payload).ok();
    }
}

impl Drop for SerialDriver {
    fn drop(&mut self) {
        // Порт закроется автоматически
    }
}
