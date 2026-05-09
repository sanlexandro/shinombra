//! Структуры для анализа результирующего цвета на основе предоставленной выборки

use crate::color::types::HSVPixel;

/// Информация о секторе
///
/// **Поля:**
/// * `weight`:[u64]        - вес
/// `votes`: [u16]          - количество голосов
/// `sum_pixel`: [HSVPixel] - сумма по всем полям для усреднения в результате
#[derive(Clone, Copy)]
pub struct ColorBin {
    pub weight: u64,

    pub votes: u16,
    pub sum_pixel: HSVPixel,
}

// Максимальное количество корзин (3 градуса - это уже на грани восприятия)
pub const MAX_BINS: usize = 120;

/// Информация о гистограмме
///
/// **Поля:**
/// - `bins`:[[ColorBin]; [MAX_BINS] + 1] - приватный массив сегментов
/// - `active_amount`: [usize] - количество сегментов, которые реально участвуют
///   в обработке
/// - `hue_to_bin_scale`: [f32] - заранее рассчитанный коэффициент для
///   определения к какой корзине относится текущий цвет
#[derive(Clone)]
pub struct ColorHistogram {
    pub(super) bins: [ColorBin; MAX_BINS + 1],
    pub(super) active_amount: usize,
    pub(super) hue_to_bin_scale: f32,
}
