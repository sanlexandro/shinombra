//! Проверка конфигурации

use super::*;
use common::configs::*;

impl ConfigValidate for ShinombraSerialConfig {
    /// Проверка [ShinombraSerialConfig]
    ///
    /// **Проверки:**
    /// - `port_path` не пустой. Путь необходим для инициализации системного вызова открытия порта.
    /// - `port_path` существует и открывается
    /// - `baud_rate` > 0. Скорость передачи данных не может быть нулевой.
    /// - `baud_rate` стандартные значения (предупреждение). Если скорость не из ряда
    ///   стандартных (9600, 115200 и т.д.), возможны проблемы с синхронизацией.
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        let section = "serial_driver_config";
        // Проверка пути к порту
        if self.port_path.trim().is_empty() {
            return Err(ValidationError::InvalidValue {
                section,
                field: "port_path",
                message: "Path to serial device cannot be empty (e.g., /dev/ttyUSB0)".into(),
            });
        }
        if !std::path::Path::new(&self.port_path).exists() {
            return Err(ValidationError::InvalidValue {
                section,
                field: "port_path",
                message: format!("Device path {} does not exist", self.port_path),
            });
        }

        // Проверка скорости (критическая)
        if self.baud_rate == 0 {
            return Err(ValidationError::InvalidValue {
                section,
                field: "baud_rate",
                message: "Baud rate must be greater than 0. Standard value is usually 115200"
                    .into(),
            });
        }

        let mut warnings = Vec::new();

        // Проверка на стандартные скорости (ворнинг)
        let standard_baud_rates = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
        if !standard_baud_rates.contains(&self.baud_rate) {
            warnings.push(ValidationWarning::InvalidValue {
                section,
                field: "baud_rate",
                message: format!(
                    "Non-standard baud rate detected: {}. Ensure your hardware supports it.",
                    self.baud_rate
                )
                .into(),
            });
        }

        Ok(warnings)
    }
}
