//! Структура обработки кадра

use std::marker::PhantomData;

use crate::{analytics::ColorAnalyst, color::types::RGBPixel, filters::ColorFilter, pixel_formatter::PixelFormatter, processing::processors::{ChunkProcessor, configs::ChunkTask}};

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
pub struct ColorEngine<Formatter, Processor, Analyst, Filter>
where
    Formatter: PixelFormatter,
    Processor: ChunkProcessor<Formatter>,
    Analyst: ColorAnalyst,
    Filter: ColorFilter,
{
    pub(crate) _formatter: PhantomData<Formatter>,
    pub(crate) processor: Processor,
    pub(crate) analyst: Analyst,
    pub(crate) filter: Filter,
    pub(crate) chunk_map: Vec<ChunkTask>,
    pub(crate) output_buffer: Vec<RGBPixel>,
}