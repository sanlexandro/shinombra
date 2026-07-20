//! Аналитик для отладки
//!
//! Получает цвет через конфиг и выводит только его

use crate::{
    analytics::{configs::DebugRGBConfig, types::DebugRGB, ColorAnalyst},
    color::types::RGBPixel,
};

impl DebugRGB {
    /// Конструкторы
    pub fn new(config: DebugRGBConfig) -> Self {
        Self {
            rgb: RGBPixel {
                red: config.red,
                green: config.green,
                blue: config.blue,
            },
        }
    }
}

impl ColorAnalyst for DebugRGB {
    type OutputFormat = RGBPixel;

    fn add_data(&mut self, _: RGBPixel) {}

    fn clear(&mut self) {}

    fn get_winner(&mut self) -> RGBPixel {
        self.rgb
    }
}
