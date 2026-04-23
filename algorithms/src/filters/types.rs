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
pub struct NoFilter {}

/// Структура для хранения фильтров
/// 
/// Данная структура необходимо для статической реализации цепи фильтров
/// 
/// Поддерживает все возможные фильтры кроме [NoFilter]
#[derive(Clone)]
pub enum FilterInstance {
    Ema(EmaFilter),
}

/// Структура для хранения цепи фильтров
/// 
/// **Поля:**
/// - `filters`: [Vec]<[FilterInstance]> - вектор хранилищ фильтров
#[derive(Clone)]
pub struct FilterChain {
    pub(super) filters: Vec<FilterInstance>,
}
