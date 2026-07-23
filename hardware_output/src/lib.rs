//! Общий интерфейс для отправки данных на устройства

pub mod debug;
pub mod wled_drgb;
pub mod events;
pub mod registry;
pub mod shinombra_serial;

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
    /// 
    /// **Выходные поля:**
    /// - [Result]<(), [HardwareEvents]> - ничего или сообщение об ошибке
    fn send_shutdown_signal(&mut self) -> Result<(), String>;
}
