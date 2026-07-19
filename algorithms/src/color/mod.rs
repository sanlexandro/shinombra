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

/// Цвет
///
/// Любой цвет должен уметь завернуть [Vec] из себя в [ColorBuffer], а также наоборот
pub trait Color: Copy + Clone + 'static + Into<RGBPixel>
where
    ColorBuffer: From<Vec<Self>> + AsMut<[Self]>,
    Vec<Self>: From<ColorBuffer>,
{
    /// Чёрный цвет
    fn black() -> Self;
}

impl Color for RGBPixel {
    fn black() -> Self {
        return RGBPixel {
            red: 0,
            green: 0,
            blue: 0,
        };
    }
}

impl Color for HSVPixel {
    fn black() -> Self {
        return HSVPixel {
            hue: 0.0,
            saturation: 0.0,
            value: 0.0,
        };
    }
}
