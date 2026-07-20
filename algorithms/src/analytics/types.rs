//! Структуры для анализа результирующего цвета на основе предоставленной выборки

use crate::color::types::{HSVPixel, RGBPixel};

/// Сектор гистограммы
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

/// Гистограмма
///
/// **Поля:**
/// - `bins`:[[ColorBin]; [MAX_BINS] + 1] - приватный массив сегментов
/// - `active_amount`: [usize] - количество сегментов, которые реально участвуют
///   в обработке
/// - `hue_to_bin_scale`: [f32] - заранее рассчитанный коэффициент для
///   определения к какой корзине относится текущий цвет
#[derive(Clone)]
pub struct Histogram {
    pub(super) bins: [ColorBin; MAX_BINS + 1],
    pub(super) active_amount: usize,
    pub(super) hue_to_bin_scale: f32,
}

/// Среднее арифметическое
///
/// **Поля:**
/// `sum_red`: [u32] - сумма по красному цвету
/// `sum_green`: [u32] - сумма по зелёному цвету
/// `sum_blue`: [u32] - сумма по синему цвету
/// `amount`: [usize] - счётчик полученных на анализ пикселей
#[derive(Clone)]
pub struct Average {
    pub(super) sum_red: u32,
    pub(super) sum_green: u32,
    pub(super) sum_blue: u32,
    pub(super) amount: usize,
}

/// Аналитик для отладки
///
/// **Поля:**
/// - `rgb`: [RGBPixel] - цвет для вывода
#[derive(Clone)]
pub struct DebugRGB {
    pub(super) rgb: RGBPixel,
}
