//! Тестирование базовых цветов по значению

use algorithms::color::types::{HSVPixel, RGBPixel};

pub mod conversions;

/// Проверка значений чёрного цвета для RGB
///
/// Все значения должны быть 0
#[test]
fn test_black_rgb() {
    let rgb = RGBPixel::black();

    assert!(rgb.red == 0, "Red in RGBPixel::black() is NOT 0!");
    assert!(rgb.green == 0, "Green in RGBPixel::black() is NOT 0!");
    assert!(rgb.blue == 0, "Blue in RGBPixel::black() is NOT 0!");
}

/// Проверка значений чёрного цвета для HSV
///
/// Все значения должны быть 0.0
#[test]
fn test_black_hsv() {
    let hsv = HSVPixel::black();

    assert!(hsv.hue == 0.0, "Hue in HSVPixel::black() is NOT 0!");
    assert!(hsv.saturation == 0.0, "Saturation in HSVPixel::black() is NOT 0!");
    assert!(hsv.value == 0.0, "Value in HSVPixel::black() is NOT 0!");
}
