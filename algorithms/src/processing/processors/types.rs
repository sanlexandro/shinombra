//! Структуры обработки фрагмента

use crate::processing::{
    configs::ScreenConfig,
    processors::configs::{CheckerboardConfig, CrawlCheckerboardConfig},
};

/// Сканер в шахматном порядке
///
/// **Поля:**
/// - `alg_config`: [CheckerboardConfig] - данные для шахматного обхода фрагмента
/// - `screen_config`: [ScreenConfig]    - информация о дисплее
pub struct CheckerboardScanner {
    pub(crate) alg_config: CheckerboardConfig,
    pub(crate) screen_config: ScreenConfig,
}

/// Сканер в динамическом шахматном порядке
///
/// **Поля:**
/// - `alg_config`: [CrawlCheckerboardConfig] - данные для шахматного обхода фрагмента
/// - `screen_config`: [ScreenConfig]    - информация о дисплее
pub struct CrawlCheckerboardScanner {
    pub(crate) alg_config: CrawlCheckerboardConfig,
    pub(crate) screen_config: ScreenConfig,
}

/// Пустое состояние
#[derive(Clone, Copy)]
pub struct EmptyState();

/// Состояние для динамической шахматки
///
/// **Поля**:
/// - `current_column`: [usize] - текущий сдвиг сетки по столбцу
/// - `current_row`: [usize]    - текущий сдвиг сетки по строке
#[derive(Copy, Clone)]
pub struct CrawlCheckerboardState {
    pub current_column: usize,
    pub current_row: usize,
}
