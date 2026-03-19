//! Общий интерфейс для отправки данных на устройства

pub mod serial;

use algorithms::color::types::RGBPixel;

use crate::serial::types::SerialDriver;

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
    fn send_colors(&mut self, colors: &[RGBPixel]);
}

/// Реализация трейта HardwareOutput для SerialDriver
impl HardwareOutput for SerialDriver {
    fn send_colors(&mut self, colors: &[RGBPixel]) {
        self.internal_send(colors);
    }
}