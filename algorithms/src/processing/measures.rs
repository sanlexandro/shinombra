//! Алгоритмы работы с различными мерами
//!
//! Здесь реализованы механизмы преобразования одних мер в другие
//!
//! Поддерживает работу с:
//! - миллиметрами [super::types::Millimeters]
//! - пикселями [super::types::Pixels]

use std::fmt;

use crate::units::*;

/// Реализация методов для Millimeters
impl Millimeters {
    /// Конструктор
    ///
    /// **Аргументы:**
    /// - `val`: [u32] - значение (миллиметры)
    pub fn new(val: u32) -> Self {
        return Self(val);
    }

    /// Преобразование миллиметров в пиксели
    ///
    /// Данный метод необходим для быстрого преобразования миллиметров в
    /// пиксели, используя вычисленный ранее коэффициент для расчёта
    ///
    /// **Аргументы:**
    /// - `mm_to_px_k`: [f64] - коэффициент преобразования миллиметров в пиксели
    pub fn as_pixels(&self, mm_to_px_k: f64) -> Pixels {
        return Pixels((self.0 as f64 * mm_to_px_k).round() as usize);
    }
}

/// Реализация отображения для [Millimeters]
impl fmt::Display for Millimeters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0) // Выводим просто число
    }
}

/// Реализация методов для Pixels
impl Pixels {
    /// Конструктор
    ///
    /// **Аргументы:**
    /// - `val`: [usize] - значение (пиксели)
    pub fn new(val: usize) -> Self {
        return Self(val);
    }

    /// Преобразование пикселей в байты
    ///
    /// Данный метод необходим для быстрого преобразования количества пикселей в
    /// количество байт
    ///
    /// **Аргументы:**
    /// - `bytes_in_px` - количество байт в одном пикселе
    pub fn as_bytes(&self, bytes_in_px: usize) -> usize {
        return self.0 * bytes_in_px;
    }
}

/// Вычисление коэффициента преобразования миллиметров в пиксели
///
/// **Аргументы:**
/// - `mm`: [Millimeters] - миллиметры
/// - `px`: [Pixels]      - пиксели
pub fn calculate_mm_to_px_k(mm: Millimeters, px: Pixels) -> f64 {
    return (px.0 as f64) / (mm.0 as f64);
}
    
/// Вычисление индекса начала пикселя в массиве байтов
///
/// Данная функция необходима для быстрого определения стартового индекса
/// пикселя в координатах (x, y)
///
/// **Аргументы:**
/// - `x`: [Pixels]          - координата x в px
/// - `y`: [Pixels]          - координата y в px
/// - `px_in_row`: [Pixels]   - количество пикселей в строке
/// - `bytes_in_px`: [usize] - количество байт в px
pub fn calculate_px_x_y_to_bytes(
    x: Pixels,
    y: Pixels,
    px_in_row: Pixels,
    bytes_in_px: usize,
) -> usize {
    return x.as_bytes(bytes_in_px) + (y.as_bytes(bytes_in_px) * px_in_row.0);
}
