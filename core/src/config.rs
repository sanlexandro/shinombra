//! Конфигурация, необходимая для непосредственной работы ядра

use std::path::PathBuf;

use algorithms::{
    analytics::registry::*, filters::registry::ColorFilterType, processing::processors::registry::*,
};
use hardware_output::registry::*;

/// Основные настройки
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

/// Флаги, влияющие на поведение программы
///
/// **Поля:**
/// - `pipewire_conversion`: [bool] - разрешить преобразование формата силами
///   pipewire
/// - `save_token`: [bool]          - разрешить сохранение токена pipewire
///   сессии, чтобы не выбирать необходимый экран при следующих запусках
#[derive(Debug, Default)]
pub struct Flags {
    pub pipewire_conversion: bool,
    pub save_token: bool,
}

/// Расположение самих настроек
///
/// Данная структура должна быть описана в отдельном файле, который и будет
/// ссылаться на основную конфигурацию и "накладки" (опционально)
///
/// **Поля:**
/// - `config`: [PathBuf]          - основной конфиг
/// - `overlays`: [Vec]<[PathBuf]> - "накладки" в порядке применения их к конфигу
pub struct Manifest {
    pub config: PathBuf,
    pub overlays: Vec<PathBuf>,
}
