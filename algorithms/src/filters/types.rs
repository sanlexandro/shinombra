//! Структуры для фильтрации цвета на основе предоставленной выборки

use crate::color::types::RGBPixel;

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
/// `precalculated_values`: [[f32]; SAMPLES_AMOUNT]        - предрассчитанные
/// значения яркости
/// `precalculated_saturations_k`: [[f32]; SAMPLES_AMOUNT] - предрассчитанные
/// коэффициенты для насыщенности
#[derive(Clone)]
pub struct BlackThreshold {
    pub threshold: f32,
    pub knee_upper_limit: f32,
    pub precalculated_values: [f32; Self::SAMPLES_AMOUNT],
    pub precalculated_saturations_k: [f32; Self::SAMPLES_AMOUNT],
}

/// Усилитель насыщения
///
/// **Поля:**
/// - `saturation_table`: [[f32]; Self::SAMPLES_AMOUNT] - LUT для насыщенности
#[derive(Clone)]
pub struct SaturationBoost {
    pub saturation_table: [f32; Self::SAMPLES_AMOUNT],
}

/// Баланс белого
///
/// **Поля:**
/// - `k_red`: [f32]   - коэффициент для красного
/// - `k_green`: [f32] - зелёного
/// - `k_blue`: [f32]  - синего
#[derive(Clone)]
pub struct WhiteBalance {
    pub k_red: f32,
    pub k_green: f32,
    pub k_blue: f32,
}

/// Ручная подстройка каналов
///
/// **Поля:**
/// - `k_red`: [f32]   - коэффициент для красного
/// - `k_green`: [f32] - зелёного
/// - `k_blue`: [f32]  - синего
#[derive(Clone)]
pub struct ChannelGain {
    pub k_red: f32,
    pub k_green: f32,
    pub k_blue: f32,
}

/// Защита от вспышек
///
/// **Поля:**
/// - `terms`: [f32]                  - LUT для добавочных значений по разнице в
///   яркости
/// - `previous_values`: [Vec]<[f32]> - предыдущие значения яркости
#[derive(Clone)]
pub struct FlashGuard {
    pub terms: [f32; Self::SAMPLES_AMOUNT],
    pub previous_values: Vec<f32>,
}

/// Хранилище фильтров
///
/// Данная структура необходимо для статической реализации цепи фильтров
///
/// Поддерживает все возможные фильтры кроме [NoFilter]
#[derive(Clone)]
pub enum FilterInstance {
    Ema(Ema),
    Gamma(Gamma),
    BlackThreshold(BlackThreshold),
    SaturationBoost(SaturationBoost),
    WhiteBalance(WhiteBalance),
    ChannelGain(ChannelGain),
    FlashGuard(FlashGuard),
}

/// Цепь фильтров
///
/// **Поля:**
/// - `filters`: [Vec]<[FilterInstance]> - вектор хранилищ фильтров
#[derive(Clone)]
pub struct FilterChain {
    pub(super) filters: Vec<FilterInstance>,
}
