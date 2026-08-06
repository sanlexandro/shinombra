use common::units::Pixels;

// Тип ориентации фрагмента
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Информация о размере фрагмента
///
/// **Поля:**
/// - `width`: [Pixels]  - ширина горизонтального фрагмента
/// - `height`: [Pixels] - высота горизонтального фрагмента
#[derive(Default)]
pub struct ChunkConfig {
    pub width: Pixels,
    pub height: Pixels,
}

/// Данные для шахматного обхода фрагмента
///
/// **Поля:**
/// - `config`: [ChunkConfig] - конфигурация фрагмента
/// - `pixel_step`: [usize]   - шаг чтения пикселей строки
/// - `row_stride`: [usize]   - шаг чтения строк
#[derive(Default)]
pub struct CheckerboardConfig {
    pub config: ChunkConfig,
    pub pixel_step: usize,
    pub row_stride: usize,
}

/// Данные для шахматного обхода фрагмента
///
/// **Поля:**
/// - `config`: [ChunkConfig] - конфигурация фрагмента
/// - `pixel_step`: [usize]   - шаг чтения пикселей строки
/// - `row_stride`: [usize]   - шаг чтения строк
/// - `column_crawl`: [usize] - шаг сдвига сетки по колонке
/// - `row_crawl`: [usize]    - шаг сдвига сетки по строке
#[derive(Default)]
pub struct CrawlCheckerboardConfig {
    pub config: ChunkConfig,
    pub pixel_step: usize,
    pub row_stride: usize,

    pub column_crawl: usize,
    pub row_crawl: usize,
}

/// Информация для обработки фрагмента
///
/// Данная структура необходима для корректной обработки фрагментов
///
/// **Поля:**
/// - `start_index`: [usize]       - начальный индекс пикселя, с которого
///   начинается фрагмент (верхний левый угол)
/// - `orientation`: [Orientation] - ориентация фрагмента
#[derive(Clone, Copy)]
pub struct ChunkTask {
    pub start_index: usize,
    pub orientation: Orientation,
}
