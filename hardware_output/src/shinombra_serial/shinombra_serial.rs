//! Реализация связи с железом по Serial
//! Простейший нативный протокол

use crate::{
    shinombra_serial::{config::ShinombraSerialConfig, types::ShinombraSerial},
    HardwareEvents, HardwareOutput,
};
use algorithms::color::types::RGBPixel;
use serialport;
use std::time::Duration;

/// Реализация методов ShinombraSerial
impl ShinombraSerial {
    /// Конструктор
    ///
    /// **Аргументы:**
    /// - `port_path`: &[str] - путь к устройству (например, "/dev/ttyUSB0")
    /// - `baud_rate`: [u32]  - скорость обмена данными
    pub fn new(config: ShinombraSerialConfig, led_amount: usize) -> Result<Self, String> {
        // Настройка порта
        let port = match serialport::new(config.port_path, config.baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
        {
            Ok(p) => p,
            Err(error) => {
                return Err(error.description);
            }
        };

        // Даем время на перезагрузку после открытия порта
        std::thread::sleep(std::time::Duration::from_secs(2));

        return Ok(Self {
            port,
            payload: Vec::with_capacity(2 + led_amount * 3),
        });
    }
}

impl HardwareOutput for ShinombraSerial {
    /// Отправка массива цвета на устройство
    ///
    /// Отправляет массив в формате `[Префикс ] + [R_1, G_1, B_1, R_2, G_2, B_2,
    /// ...]`
    ///
    /// В качестве префикса было выбрано слово "AD" (AmbiData)
    ///
    /// **Поля:**
    /// - `colors`: &[[RGBPixel]] - массив из RGBPixel
    fn send_colors(&mut self, colors: &[RGBPixel]) -> Result<(), HardwareEvents> {
        // Формируем пакет: [Префикс] + [RGB данные]
        self.payload.clear();
        self.payload.extend_from_slice(b"AD"); // Magic Word (AmbiData)

        // Сохраняем для отправки
        for color in colors {
            self.payload
                .extend_from_slice(&[color.red, color.green, color.blue]);
        }

        // Отправляем всё одним махом
        match self.port.write_all(&self.payload) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    /// Отправка сигнала завершения
    ///
    /// Отправляет на устройство сигнал завершения - слово "SD" (ShutDown)
    fn send_shutdown_signal(&mut self) -> Result<(), String> {
        match self.port.write_all(b"TERM") {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("Failed to send TERM signal via serial: {}", error)),
        }
    }
}

impl Drop for ShinombraSerial {
    fn drop(&mut self) {
        // Порт закроется автоматически
    }
}
