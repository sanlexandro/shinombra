//! Тестирование алгоритмов преобразования форматов
//!
//! Здесь тестируются преобразования цветов из одного в другой тип, а также
//! проверка, что при обратном преобразовании исходный цвет не меняется

use crate::test_fns::*;
use algorithms::color::{conversion::*, types::*};

/// Проверка преобразования из RGB в HSV
///
/// Данный тест проверяет корректность математического перевода цвета
/// из цветовой модели RGB в HSV.
///
/// **Поля таблицы:**
/// - `rgb` - исходный цвет в формате RGB
/// - `expected_h` - ожидаемое значение тона (в градусах)
/// - `expected_s` - ожидаемое значение насыщенности (0.0..1.0)
/// - `expected_v` - ожидаемое значение яркости (0.0..255.0)
#[test]
fn test_rgb_to_hsv() {
    let cases = vec![
        // (rgb, expected_h, expected_s, expected_v)
        (RGBPixel::black(), 0.0, 0.0, 0.0), // Чёрный цвет
        (
            RGBPixel {
                red: 255,
                green: 0,
                blue: 0,
            },
            0.0,
            1.0,
            255.0,
        ), // Красный цвет (угол 0)
    ];

    for (rgb, expected_h, expected_s, expected_v) in cases {
        let hsv = convert_rgb_to_hsv(rgb.clone());

        assert_approximately_equal(hsv.hue, expected_h, 0.001);
        assert_approximately_equal(hsv.saturation, expected_s, 0.001);
        assert_approximately_equal(hsv.value, expected_v, 0.001);
    }
}

/// Проверка преобразования из HSV в RGB
///
/// Данный тест проверяет корректность математического перевода цвета
/// из цветовой модели HSV в RGB.
///
/// **Поля таблицы:**
/// - `hsv` - исходный цвет в формате HSV
/// - `expected_r` - ожидаемое значение красного
/// - `expected_g` - ожидаемое значение зеленого
/// - `expected_b` - ожидаемое значение синего
#[test]
fn test_hsv_to_rgb() {
    let cases = vec![
        // (hsv, expected_r, expected_g, expected_b)
        // Зелёный цвет (в HSV находится на угле 120 градусов)
        (
            HSVPixel {
                hue: 120.0,
                saturation: 1.0,
                value: 255.0,
            },
            0,
            255,
            0,
        ),
    ];

    for (hsv, expected_r, expected_g, expected_b) in cases {
        let rgb = convert_hsv_to_rgb(hsv);

        assert_eq!(rgb.red, expected_r);
        assert_eq!(rgb.green, expected_g);
        assert_eq!(rgb.blue, expected_b);
    }
}

/// Проверка обратимости преобразования (RGB -> HSV -> RGB)
///
/// Данный тест проверяет, что если перевести цвет RGB в HSV и потом
/// конвертировать обратно, то мы получим точно такой же цвет без
/// искажений в результате вычислений.
///
/// **Поля таблицы:**
/// - `original_rgb` - исходный цвет в формате RGB
#[test]
fn test_conversion_reversibility() {
    let cases = vec![
        // original_rgb
        RGBPixel {
            red: 100,
            green: 150,
            blue: 200,
        },
    ];

    for original_rgb in cases {
        let hsv = convert_rgb_to_hsv(original_rgb.clone());
        let back_to_rgb = convert_hsv_to_rgb(hsv);

        assert_eq!(original_rgb.red, back_to_rgb.red);
        assert_eq!(original_rgb.green, back_to_rgb.green);
        assert_eq!(original_rgb.blue, back_to_rgb.blue);
    }
}
