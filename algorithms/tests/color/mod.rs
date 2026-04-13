//! Тестирование основных методов форматов
//!
//! Здесь тестируются основные методы форматов, а именно - заданные цвета

use algorithms::color::types::*;
pub mod conversion;

/// Проверка корректности конструкторов базовых цветов в RGB формате
///
/// Данный тест проверяет правильность инициализации константных цветов.
///
/// **Поля таблицы:**
/// - `color` - сгенерированный цвет (RGBPixel)
/// - `expected_r` - ожидаемое значение красного (Red)
/// - `expected_g` - ожидаемое значение зеленого (Green)
/// - `expected_b` - ожидаемое значение синего (Blue)
#[test]
fn test_rgb_base_colors() {
    let cases = vec![
        // (color, expected_r, expected_g, expected_b)
        (RGBPixel::black(), 0, 0, 0),
    ];

    for (color, expected_r, expected_g, expected_b) in cases {
        assert_eq!(color.red, expected_r);
        assert_eq!(color.green, expected_g);
        assert_eq!(color.blue, expected_b);
    }
}

/// Проверка корректности конструкторов базовых цветов в HSV формате
///
/// Данный тест проверяет правильность инициализации константных цветов.
///
/// **Поля таблицы:**
/// - `color` - сгенерированный цвет (HSVPixel)
/// - `expected_h` - ожидаемое значение тона (Hue)
/// - `expected_s` - ожидаемое значение насыщенности (Saturation)
/// - `expected_v` - ожидаемое значение яркости (Value)
#[test]
fn test_hsv_base_colors() {
    let cases = vec![
        // (color, expected_h, expected_s, expected_v)
        (HSVPixel::black(), 0.0, 0.0, 0.0),
    ];

    for (color, expected_h, expected_s, expected_v) in cases {
        assert_eq!(color.hue, expected_h);
        assert_eq!(color.saturation, expected_s);
        assert_eq!(color.value, expected_v);
    }
}
