//! Модуль для работы с различными форматами цвета
//!
//! Предоставляет возможность хранить и конвертировать разные форматы
//!
//! **Поддерживает форматы:**
//! - RGB *(Red Green Blue)*
//! - HSV *(Hue Saturation Value)*

pub mod conversion;
pub mod types;

use crate::color::types::*;

/// Реализация стандартных методов для RGBPixel
impl RGBPixel {
    /// Чёрный цвет
    /// 
    /// **Выходные данные:**
    /// - чёрный пиксель в [RGBPixel]
    pub fn black() -> Self {
        return RGBPixel {
            red: 0,
            green: 0,
            blue: 0,
        };
    }
}

/// Реализация стандартных методов для HSVPixel
impl HSVPixel {
    /// Чёрный цвет
    /// 
    /// **Выходные данные:**
    /// - чёрный пиксель в [HSVPixel]
    pub fn black() -> Self {
        return HSVPixel {
            hue: 0.0,
            saturation: 0.0,
            value: 0.0,
        };
    }
}
