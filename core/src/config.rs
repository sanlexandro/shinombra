//! Конфигурация, необходимая для непосредственной работы ядра

use std::path::PathBuf;

use algorithms::{
    analytics::registry::*, filters::registry::ColorFilterType, processing::processors::registry::*,
};
use hardware_output::registry::*;
use logger::LogLevel;

/// Основные настройки
///
/// **Поля:**
/// - `chunk_processor_type`: [ChunkProcessorType] - тип обработчика фрагмента
/// - `analytics_type`: [ColorAnalystType]         - тип анализатора цвета
/// - `filter_chain`: [Vec]<[ColorFilterType]>     - цепь фильтров
/// - `hardware_output_type`: [HardwareOutputType] - тип вывода
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
#[derive(Default)]
pub struct Flags {
    pub pipewire_conversion: bool,
    pub save_token: bool,
}

/// Пути до файлов для манифеста
///
/// Данная структура должна быть описана в отдельном файле, который и будет
/// ссылаться на основную конфигурацию и "накладки" (опционально)
///
/// **Поля:**
/// - `config`: [PathBuf]          - основной конфиг
/// - `overlays`: [Vec]<[PathBuf]> - "накладки" в порядке применения их к конфигу
pub struct Paths {
    pub config: PathBuf,
    pub overlays: Vec<PathBuf>,
}

/// Настройки самого процесса
///
/// Содержат настройки поведения самого процесса
///
/// **Поля:**
/// - `log_level`: [LogLevel] - уровень логирования
pub struct DaemonSettings {
    pub log_level: LogLevel,
}

/// Манифест
///
/// Содержит в себя глобальные настройки процесса и пути до конфигурации подсветки
///
/// **Поля:**
/// `paths`: [Paths]                    - пути до конфигов
/// `daemon_settings`: [DaemonSettings] - настройки процесса
pub struct Manifest {
    pub paths: Option<Paths>,
    pub daemon_settings: Option<DaemonSettings>,
}
