//! Структуры обработки кадра и фрагментов

use crate::{analytics::ColorAccumulator, processing::ChunkProcessor};

/// Информация о дисплее
///
/// **Поля:**
/// - `frame_width`: [usize]  - ширина кадра
/// - `frame_height`: [usize] - высота кадра
pub struct ScreenConfig {
    pub frame_width: usize,  // ширина кадра
    pub frame_height: usize, // высота кадра
}

/// Тип ориентации фрагмента
#[derive(PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Данные для шахматного обхода фрагмента
///
/// **Поля:**
/// - `chunk_width`: [usize]  - ширина фрагмента
/// - `chunk_height`: [usize] - высота фрагмента
/// - `pixel_step`: [usize]   - шаг чтения пикселей строки
/// - `row_stride`: [usize]   - шаг чтения строк
pub struct CheckerboardConfig {
    pub chunk_width: usize,  // ширина фрагмента
    pub chunk_height: usize, // высота фрагмента
    pub pixel_step: usize,   // шаг чтения пикселей строки
    pub row_stride: usize,   // шаг чтения строк
}

/// Сканер в шахматном порядке
///
/// **Поля:**
/// - `alg_config`: [CheckerboardConfig] - данные для шахматного обхода фрагмента
/// - `screen_config`: [ScreenConfig]    - информация о дисплее
pub struct CheckerboardScanner {
    pub(crate) alg_config: CheckerboardConfig,
    pub(crate) screen_config: ScreenConfig,
}

/// Движок обработки кадров
///
/// Вызывается при обработке каждого кадра. Самостоятельно разделяет кадр на
/// фрагменты, вызывает обработку фрагментов, вызывает анализ результирующего
/// цвета
///
/// **Поля:**
/// - `processor`: [ChunkProcessor]     - метод обхода фрагмента
/// - `accumulator`: [ColorAccumulator] - метод анализа цвета в фрагменте
pub struct ColorEngine<P, A>
where
    P: ChunkProcessor,
    A: ColorAccumulator,
{
    pub(crate) processor: P,
    pub(crate) accumulator: A, // TODO: сделать вектор гистограмм
}