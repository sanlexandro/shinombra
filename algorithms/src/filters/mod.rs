//! Модуль для постобработки цветов
//!
//! Данный модуль реализует сглаживание между кадрами, а также может в
//! дальнейшем реализовать некоторую логику изменения цвета изображения после
//!
//! **Методы:**
//! - Отсутствие фильтра [NoFilter]
//! - Экспоненциальное скользящее среднее [EmaFilter] - смешивает прошлый цвет и
//!   текущий вычисленный в пропорции 3:1
//! - Гамма фильтр [GammaFilter] - обеспечивает логорифмическое преобразование яркости
//!
//! Также данный модуль поддерживает запуск цепи из фильтров [FilterChain]

pub mod chain;
pub mod configs;
pub mod ema;
pub mod gamma;
pub mod no_filter;
pub mod registry;
pub mod types;

use crate::{
    color::types::RGBPixel,
    filters::types::{EmaFilter, FilterChain, FilterInstance, GammaFilter, NoFilter},
};

/// Трейт постобработки цвета
///
/// **Методы:**
pub trait ColorFilter: Clone {
    /// Вычисление результирующего цвета после фильтрации
    ///
    /// **Поля:**
    /// - `raw_colors`: &[[RGBPixel]] - массив новых пикселей для наложения фильтра
    fn apply(&mut self, raw_colors: &mut [RGBPixel]);
}

/// Реализация трейта [ColorFilter] для хранилища фильтров [FilterInstance]
impl ColorFilter for FilterInstance {
    fn apply(&mut self, raw_colors: &mut [RGBPixel]) {
        match self {
            Self::Ema(f) => f.apply(raw_colors),
            Self::Gamma(f) => f.apply(raw_colors),
        }
    }
}

/// Реализация трейта [ColorFilter] для [EmaFilter]
impl ColorFilter for EmaFilter {
    fn apply(&mut self, raw_colors: &mut [RGBPixel]) {
        self.process_ema(raw_colors);
    }
}

/// Реализация трейта [ColorFilter] для [NoFilter]
impl ColorFilter for NoFilter {
    fn apply(&mut self, _raw_colors: &mut [RGBPixel]) {}
}

impl ColorFilter for GammaFilter {
    fn apply(&mut self, raw_colors: &mut [RGBPixel]) {
        self.apply_gamma(raw_colors);
    }
}

/// Реализация трейта [ColorFilter] для [FilterChain]
impl ColorFilter for FilterChain {
    fn apply(&mut self, raw_colors: &mut [RGBPixel]) {
        self.run_chain(raw_colors);
    }
}
