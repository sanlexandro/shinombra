//! Структуры обработки кадра и фрагментов
//!
use super::configs::*;
use crate::{analytics::ColorAccumulator, color::types::RGBPixel, processing::ChunkProcessor};

/// Движок обработки кадров
///
/// Вызывается при обработке каждого кадра. Самостоятельно разделяет кадр на
/// фрагменты, вызывает обработку фрагментов, вызывает анализ результирующего
/// цвета
///
/// **Поля:**
/// - `processor`: [ChunkProcessor]     - метод обхода фрагмента
/// - `accumulator`: [ColorAccumulator] - метод анализа цвета в фрагменте
/// - `chunk_map`: [ChunkTask]          - карта фрагментов (рассчитывается при
///   инициализации)
/// - `output_buffer`: [Vec<RGBPixel>]  - вектор с вычисленными результатами
pub struct ColorEngine<P, A>
where
    P: ChunkProcessor,
    A: ColorAccumulator,
{
    pub(crate) processor: P,
    pub(crate) accumulator: A, // TODO: сделать вектор гистограмм
    pub(crate) chunk_map: Vec<ChunkTask>,
    pub(crate) output_buffer: Vec<RGBPixel>,
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
