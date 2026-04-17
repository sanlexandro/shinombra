//! Структуры для фильтрации цвета на основе предоставленной выборки

use crate::color::types::RGBPixel;

/// Структура для хранений накопленного экспоненциального скользящего среднего
///
/// **Поля:**
/// - `states`: Vec<[RGBPixel]> - вектор последних отфильтрованных цветов
#[derive(Clone)]
pub struct EmaFilter {
    pub(super) states: Vec<RGBPixel>,
}

/// Структура для "работы" отсутствия фильтра
#[derive(Clone)]
pub struct NoFilter {
    pub(super) states: Vec<RGBPixel>,
}
