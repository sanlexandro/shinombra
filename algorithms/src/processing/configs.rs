use crate::{processing::measures::calculate_mm_to_px_k, units::*};

/// Информация о дисплее
///
/// **Поля:**
/// - `frame_width`: [Pixels]  - ширина кадра в пикселях
/// - `frame_height`: [Pixels] - высота кадра в пикселях
#[derive(Clone, Copy)]
pub struct ScreenConfig {
    pub frame_width_px: Pixels,
    pub frame_height_px: Pixels,
    pub frame_width_mm: Millimeters,
    pub frame_height_mm: Millimeters,
}

/// Информация о физическом расположении ленты
///
/// Данная структура хранит данные о расположении ленты относительно монитора, а
/// также некоторые физические параметры самой ленты
///
/// Алгоритм предполагает, что кусочки ленты наклеены ровно по центру каждой
/// грани, а также, что все кусочки имеют приблизительно одинаковое расстояние
/// до края экрана
///
/// **Поля:**
/// - `gap`: [Millimeters]               - расстояние от центра ленты (т.е.
///   приблизительно от центра самого диода) до края экрана
/// - `vertical_offset`: [Millimeters]   - расстояние от края вертикальной ленты
///   до края экрана
/// - `horizontal_offset`: [Millimeters] - расстояние от края горизонтальной
///   ленты до края экрана
/// - `led_length`: [Millimeters]        - длина одного блока светодиодов
/// - `vertical_led_amount`: [u8]        - количество блоков светодиодов на
///   вертикальной ленте
/// - `horizontal_led_amount`: [u8]      - количество блоков светодиодов на
///   горизонтальной ленте
///
/// Под блоком светодиодов подразумевается n светодиодов (n > 0), которые
/// физически соединены на ленте и не могут управляться отдельно друг от друга
pub struct LedPositionConfig {
    pub gap: Millimeters,
    pub vertical_offset: Millimeters,
    pub horizontal_offset: Millimeters,
    pub led_length: Millimeters,

    pub vertical_led_amount: usize,
    pub horizontal_led_amount: usize,
}

/// Информация о глубине чтения экрана
///
/// **Поля:** всё в [Millimeters]
/// - `deep_in`  - глубина чтения к центру экрана от центра ленты
/// - `deep_out` - глубина чтения экрана к краям экрана от центра ленты
pub struct ScreenReadingConfig {
    pub deep_in: Millimeters,
    pub deep_out: Millimeters,
}

/// Информация о физическом положении летны и настройках
///
/// **Поля:**
/// - `led_pos`: [LedPositionConfig]   - информация о физическом расположении
///   ленты
/// - `reading`: [ScreenReadingConfig] - информация о настройках чтения дисплея
pub struct GeometryConfig {
    pub led_pos: LedPositionConfig,
    pub reading: ScreenReadingConfig,
}

/// Информация о размере фрагмента
///
/// **Поля:**
/// - `width`: [Pixels]  - ширина горизонтального фрагмента
/// - `height`: [Pixels] - высота горизонтального фрагмента
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
pub struct CheckerboardConfig {
    pub config: ChunkConfig,
    pub pixel_step: usize,
    pub row_stride: usize,
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

// Реализация методов GeometryConfig
impl GeometryConfig {
    /// Расчёт конфигурации фрагмента
    ///
    /// Данный метод необходим для определения конфигурации фрагмента, исходя из
    /// данных, переданных пользователю
    ///
    /// **Аргументы:**
    /// - `screen_config`: [ScreenConfig] - информация об экране
    pub fn calculate_chunk_config(&self, screen_config: ScreenConfig) -> ChunkConfig {
        // Коэффициент по вертикали
        let k_y =
            calculate_mm_to_px_k(screen_config.frame_height_mm, screen_config.frame_height_px);
        // Коэффициент по горизонтали (на случай нестандартных экранов)
        let k_x = calculate_mm_to_px_k(screen_config.frame_width_mm, screen_config.frame_width_px);

        ChunkConfig {
            // Ширина — это длина блока диодов (в px)
            width: self.led_pos.led_length.as_pixels(k_x),
            // Высота — это суммарная глубина захвата (в px)
            height: Pixels::new(
                ((self.reading.deep_in.0 + self.reading.deep_out.0) as f64 * k_y).round() as usize,
            ),
        }
    }
}
