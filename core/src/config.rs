//! Конфигурация, необходимая для непосредственной работы ядра

use algorithms::{analytics::registry::*, filters::registry::ColorFilterType, processing::registry::*};
use hardware_output::registry::*;

/// Структура для хранения основных критических настроек
/// 
/// **Поля:**
/// - `chunk_processor_type`: [ChunkProcessorType] - тип обработчика фрагмента 
/// - `analytics_type`: [ColorAnalystType]         - тип анализатора цвета
/// - `filter_chain`: [Vec]<[ColorFilterType]>     - цепь фильтров
/// - `hardware_output_type`: [HardwareOutputType] - тип вывода
#[derive(Debug)]
pub struct Settings {
    pub chunk_processor_type: ChunkProcessorType,
    pub analytics_type: ColorAnalystType,
    pub filter_chain: Vec<ColorFilterType>,
    pub hardware_output_type: HardwareOutputType,
}