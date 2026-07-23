//! Реализация связи с железом по Serial
//! Простейший нативный протокол

use crate::{
    shinombra::{config::ShinombraConfig, types::Shinombra},
    HardwareEvents, HardwareOutput,
};
use algorithms::color::types::RGBPixel;
use serialport;
use std::time::Duration;

/// Реализация методов Shinombra
impl Shinombra {
    /// Конструктор
    ///
    /// **Аргументы:**
    /// - `port_path`: &[str] - путь к устройству (например, "/dev/ttyUSB0")
    /// - `baud_rate`: [u32]  - скорость обмена данными
    pub fn new(config: ShinombraConfig) -> Result<Self, String> {
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

        return Ok(Self { port });
    }
}

impl HardwareOutput for Shinombra {
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

impl Drop for Shinombra {
    fn drop(&mut self) {
        // Порт закроется автоматически
    }
}
