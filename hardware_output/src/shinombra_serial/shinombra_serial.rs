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

    /// Блокирующее ожидание синхронизации
    pub fn wait_for_ready(&mut self) -> Result<(), HardwareEvents> {
        let mut buf = [0u8; 6]; // "READY\n"

        // Пытаемся прочитать 6 байт из Serial порта
        match self.port.read_exact(&mut buf) {
            Ok(_) => {
                if &buf == b"READY\n" || &buf[..5] == b"READY" {
                    Ok(())
                } else {
                    // Если прилетел какой-то другой мусор вместо READY -
                    // просим движок переотправить/подождать
                    Err(HardwareEvents::RetryNeeded)
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Вышли по таймауту - плата еще занята
                Err(HardwareEvents::RetryNeeded)
            }
            Err(e) => {
                // Какая-то критическая ошибка порта (кабель выдернули)
                Err(e.into())
            }
        }
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
        // Ждём, пока контроллер сообщит о готовности
        self.wait_for_ready()?;

        // Формируем пакет
        self.payload.clear();
        self.payload.extend_from_slice(b"AD"); // Magic Word (AmbiData)

        for color in colors {
            self.payload
                .extend_from_slice(&[color.red, color.green, color.blue]);
        }

        // Отправляем всё в порт
        match self.port.write_all(&self.payload) {
            Ok(_) => {
                // Сбрасываем буфер записи, чтобы байты ушли в кабель моментально
                let _ = self.port.flush();
                Ok(())
            }
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
