//! Структуры для фильтрации цвета на основе предоставленной выборки

use crate::{color::types::RGBPixel, filters::black_threshold::SAMPLES_AMOUNT};

/// EMA-фильтр
///
/// **Поля:**
/// - `states`: Vec<[RGBPixel]> - вектор последних отфильтрованных цветов
#[derive(Clone)]
pub struct Ema {
    pub(super) states: Vec<RGBPixel>,
    pub(super) alpha: i32,
    pub(super) inverted_alpha: i32,
}

/// Структура для "работы" отсутствия фильтра
#[derive(Clone)]
pub struct NoFilter {}

/// Гамма-корректор
///
/// Расчёт корректных значений до запуска позволит избавиться от постоянного
/// вычисления степеней на лету
///
/// **Поля:**
/// - `gamma_table`: [[u8]; 256] - статический массив с рассчитанными значениями
///   (всегда 256 в силу характеристики самого [RGBPixel])
#[derive(Clone)]
pub struct Gamma {
    pub(super) gamma_table: [u8; 256],
}

/// Отсекатель тусклого
///
/// **Поля:**
/// `threshold`: [f32]                                     - нижний порог отсечки
/// `knee_upper_limit`: [f32]                              - верхний порог
/// `precalculated_values`: [[f32]; SAMPLES_AMOUNT]        - предрасчитанные
/// значения яркости
/// `precalculated_saturations_k`: [[f32]; SAMPLES_AMOUNT] - предрасчитанные
/// коэффициенты для насыщенности
#[derive(Clone)]
pub struct BlackThreshold {
    pub threshold: f32,
    pub knee_upper_limit: f32,
    pub precalculated_values: [f32; SAMPLES_AMOUNT],
    pub precalculated_saturations_k: [f32; SAMPLES_AMOUNT],
}

/// Структура для хранения фильтров
///
/// Данная структура необходимо для статической реализации цепи фильтров
///
/// Поддерживает все возможные фильтры кроме [NoFilter]
#[derive(Clone)]
pub enum FilterInstance {
    Ema(Ema),
    Gamma(Gamma),
    BlackThreshold(BlackThreshold),
}

/// Структура для хранения цепи фильтров
///
/// **Поля:**
/// - `filters`: [Vec]<[FilterInstance]> - вектор хранилищ фильтров
#[derive(Clone)]
pub struct FilterChain {
    pub(super) filters: Vec<FilterInstance>,
}
