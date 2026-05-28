//! Структуры обработки фрагмента

use crate::processing::{
    configs::ScreenConfig,
    processors::configs::{CheckerboardConfig, DynamicCheckerboardConfig},
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
/// - `alg_config`: [DynamicCheckerboardConfig] - данные для шахматного обхода фрагмента
/// - `screen_config`: [ScreenConfig]    - информация о дисплее
pub struct DynamicCheckerboardScanner {
    pub(crate) alg_config: DynamicCheckerboardConfig,
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
pub struct DynamicCheckerboardState {
    pub current_column: usize,
    pub current_row: usize,
}
