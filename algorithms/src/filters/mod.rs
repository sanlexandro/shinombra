//! Модуль для постобработки цветов
//!
//! Данный модуль реализует сглаживание между кадрами, а также может в
//! дальнейшем реализовать некоторую логику изменения цвета изображения после
//!
//! **Методы:**
//! - Экспоненциальное скользящее среднее [EmaFilter] - смешивает прошлый цвет и
//!   текущий вычисленный в пропорции 3:1

pub mod ema;
pub mod registry;
pub mod types;

use crate::{color::types::RGBPixel, filters::types::EmaFilter};

/// Трейт постобработки цвета
///
/// **Методы:**
pub trait ColorFilter: Clone {
    /// Вычисление результирующего цвета после фильтрации
    ///
    /// **Поля:**
    /// - `raw_colors`: &[[RGBPixel]] - массив новых пикселей для наложения фильтра
    fn apply(&mut self, raw_colors: &[RGBPixel]) -> &[RGBPixel];
}

/// Реализация трейта [ColorFilter] для [EmaFilter]
impl ColorFilter for EmaFilter {
    fn apply(&mut self, raw_colors: &[RGBPixel]) -> &[RGBPixel] {
        self.process_ema(raw_colors)
    }
}