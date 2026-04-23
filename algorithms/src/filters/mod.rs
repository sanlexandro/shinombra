//! Модуль для постобработки цветов
//!
//! Данный модуль реализует сглаживание между кадрами, а также может в
//! дальнейшем реализовать некоторую логику изменения цвета изображения после
//!
//! **Методы:**
//! - Экспоненциальное скользящее среднее [EmaFilter] - смешивает прошлый цвет и
//!   текущий вычисленный в пропорции 3:1
//! - Отсутствие фильтра [NoFilter]
//!
//! Также данный модуль поддерживает запуск цепи из фильтров [FilterChain]

pub mod chain;
pub mod ema;
pub mod no_filter;
pub mod registry;
pub mod types;

use crate::{
    color::types::RGBPixel,
    filters::types::{EmaFilter, FilterChain, FilterInstance, NoFilter},
};

/// Трейт постобработки цвета
///
/// **Методы:**
pub trait ColorFilter: Clone {
    /// Вычисление результирующего цвета после фильтрации
    ///
    /// **Поля:**
    /// - `raw_colors`: &[[RGBPixel]] - массив новых пикселей для наложения фильтра
    fn apply<'a>(&'a mut self, raw_colors: &'a [RGBPixel]) -> &'a [RGBPixel];
}

/// Реализация трейта [ColorFilter] для хранилища фильтров [FilterInstance]
impl ColorFilter for FilterInstance {
    fn apply<'a>(&'a mut self, raw_colors: &'a [RGBPixel]) -> &'a [RGBPixel] {
        match self {
            Self::Ema(f) => f.apply(raw_colors),
        }
    }
}

/// Реализация трейта [ColorFilter] для [EmaFilter]
impl ColorFilter for EmaFilter {
    fn apply<'a>(&'a mut self, raw_colors: &'a [RGBPixel]) -> &'a [RGBPixel] {
        self.process_ema(raw_colors)
    }
}

/// Реализация трейта [ColorFilter] для [NoFilter]
impl ColorFilter for NoFilter {
    fn apply<'a>(&'a mut self, raw_colors: &'a [RGBPixel]) -> &'a [RGBPixel] {
        raw_colors
    }
}

/// Реализация трейта [ColorFilter] для [FilterChain]
impl ColorFilter for FilterChain {
    fn apply<'a>(&'a mut self, raw_colors: &'a [RGBPixel]) -> &'a [RGBPixel] {
        self.run_chain(raw_colors)
    }
}
