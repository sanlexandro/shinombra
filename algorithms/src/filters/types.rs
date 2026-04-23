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

/// Структура для работы гамма-коррекции
/// 
/// Расчёт корректных значений до запуска позволит избавиться от постоянного
/// вычисления степеней на лету
/// 
/// **Поля:**
/// - `gamma_table`: [[u8]; 256] - статический массив с рассчитанными значениями
///   (всегда 256 в силу характеристики самого [RGBPixel])
#[derive(Clone)]
pub struct GammaFilter {
    pub(super) gamma_table: [u8; 256],
}

/// Структура для хранения фильтров
///
/// Данная структура необходимо для статической реализации цепи фильтров
///
/// Поддерживает все возможные фильтры кроме [NoFilter]
#[derive(Clone)]
pub enum FilterInstance {
    Ema(EmaFilter),
    Gamma(GammaFilter),
}

/// Структура для хранения цепи фильтров
///
/// **Поля:**
/// - `filters`: [Vec]<[FilterInstance]> - вектор хранилищ фильтров
#[derive(Clone)]
pub struct FilterChain {
    pub(super) filters: Vec<FilterInstance>,
}
