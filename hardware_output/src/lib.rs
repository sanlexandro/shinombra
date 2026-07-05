//! Общий интерфейс для отправки данных на устройства

pub mod debug;
pub mod registry;
pub mod serial;

use algorithms::color::types::RGBPixel;

/// События устройства
pub enum HardwareEvents {
    Disconnected,
    RetryNeeded,
    NoAccess,
    InternalError(String),
}

/// Трейт отправки данных на устройства
///
/// **Методы:**
/// - `send_colors` - отправить массив цветов на устройство
pub trait HardwareOutput {
    /// Отправить массив цветов на устройство
    ///
    /// Данный метод необходим для отправки готового массива цветов на отрисовку
    /// на устройстве
    ///
    /// **Аргументы:**
    /// - `colors`: &[[RGBPixel]] - массив [RGBPixel]
    ///
    /// **Выходные поля:**
    /// - [Result]<(), [HardwareEvents]> - ничего или код события
    fn send_colors(&mut self, colors: &[RGBPixel]) -> Result<(), HardwareEvents>;

    /// Отправить сигнал завершения на устройство
    fn send_shutdown_signal(&mut self) -> Result<(), String>;
}
