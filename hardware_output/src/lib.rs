//! Общий интерфейс для отправки данных на устройства

pub mod serial;
pub mod debug;

use algorithms::color::types::RGBPixel;

use crate::{debug::types::DebugDriver, serial::types::SerialDriver};

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

/// Реализация трейта HardwareOutput для DebugDriver
impl HardwareOutput for DebugDriver {
    fn send_colors(&mut self, colors: &[RGBPixel]) {
        self.print_debug_frame(colors);
    }
}