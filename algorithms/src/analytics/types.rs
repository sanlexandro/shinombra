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

/// Информация о гистограмме
///
/// **Поля:**
/// - `bins`:[[ColorBin]; 37] - приватный массив сегментов
#[derive(Clone)]
pub struct ColorHistogram {
    pub(super) bins: [ColorBin; 37], // TODO: реализовать считывание кол-ва сегментов из конфига
}
