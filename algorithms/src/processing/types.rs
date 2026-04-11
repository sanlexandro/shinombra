//! Структуры обработки кадра и фрагментов
//!
use super::configs::*;
use crate::{analytics::ColorAnalyst, color::types::RGBPixel, filters::ColorFilter, processing::ChunkProcessor};

/// Движок обработки кадров
///
/// Вызывается при обработке каждого кадра. Самостоятельно разделяет кадр на
/// фрагменты, вызывает обработку фрагментов, вызывает анализ результирующего
/// цвета
///
/// **Поля:**
/// - `processor`: [ChunkProcessor]     - метод обхода фрагмента
/// - `analyst`: [ColorAnalyst]         - метод анализа цвета в фрагменте
/// - `filter`: [ColorFilter]           - метод фильтрации результатов анализа
/// - `chunk_map`: [ChunkTask]          - карта фрагментов (рассчитывается при
///   инициализации)
/// - `output_buffer`: [Vec<RGBPixel>]  - вектор с вычисленными результатами
pub struct ColorEngine<Processor, Analyst, Filter>
where
    Processor: ChunkProcessor,
    Analyst: ColorAnalyst,
    Filter: ColorFilter,
{
    pub(crate) processor: Processor,
    pub(crate) analyst: Analyst,
    pub(crate) filter: Filter,
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
