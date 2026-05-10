use common::units::*;


/// Информация о дисплее
///
/// **Поля:**
/// - `frame_width`: [Pixels]  - ширина кадра в пикселях
/// - `frame_height`: [Pixels] - высота кадра в пикселях
#[derive(Default, Clone, Copy, Debug)]
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
#[derive(Default, Debug, Clone, Copy)]
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
#[derive(Default, Debug, Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct GeometryConfig {
    pub led_pos: LedPositionConfig,
    pub reading: ScreenReadingConfig,
}