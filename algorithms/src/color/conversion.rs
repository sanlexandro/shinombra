//! Алгоритмы для преобразования форматов

use super::types::{HSVPixel, RGBPixel};

/// Преобразование RGB формата в HSV
///
/// **Аргументы:**
/// - `rgb`:[RGBPixel] - пиксель в формате RGB, который требуется преобразовать
///
/// **Выходные данные:**
/// - [HSVPixel] - HSV формате
pub fn convert_rgb_to_hsv(rgb: RGBPixel) -> HSVPixel {
    let max = rgb.blue.max(rgb.green).max(rgb.red) as f32;
    let min = rgb.blue.min(rgb.green).min(rgb.red) as f32;

    let delta = max - min;

    let value = max;

    let saturation = if max == 0.0 { 0.0 } else { delta / max };

    let mut hue = if delta == 0.0 {
        0.0
    } else if max == rgb.red as f32 {
        // Приводим К f32 ДО вычитания
        60.0 * (((rgb.green as f32 - rgb.blue as f32) / delta) % 6.0)
    } else if max == rgb.green as f32 {
        60.0 * (((rgb.blue as f32 - rgb.red as f32) / delta) + 2.0)
    } else {
        60.0 * (((rgb.red as f32 - rgb.green as f32) / delta) + 4.0)
    };

    if hue < 0.0 {
        hue += 360.0;
    }

    // Возвращаем значение
    return HSVPixel {
        hue,
        saturation,
        value,
    };
}

/// Преобразование HSV формата в RGB
///
/// **Аргументы:**
/// - `hsv`:[HSVPixel] - пиксель в формате HSV, который требуется преобразовать
///
/// **Выходные данные:**
/// - [RGBPixel] - пиксель в RGB формате
pub fn convert_hsv_to_rgb(hsv: HSVPixel) -> RGBPixel {
    let chroma = hsv.value * hsv.saturation;
    let x = chroma * (1.0 - ((hsv.hue / 60.0) % 2.0 - 1.0).abs());
    let m = hsv.value - chroma;

    // В зависимости от сектора (никак не связано с секторами, на которые мы
    // разбили круг. Это связано с самой логикой HSV формата) преобразуем цвет
    let (r_prime, g_prime, b_prime) = if hsv.hue < 60.0 {
        (chroma, x, 0.0)
    } else if hsv.hue < 120.0 {
        (x, chroma, 0.0)
    } else if hsv.hue < 180.0 {
        (0.0, chroma, x)
    } else if hsv.hue < 240.0 {
        (0.0, x, chroma)
    } else if hsv.hue < 300.0 {
        (x, 0.0, chroma)
    } else {
        (chroma, 0.0, x)
    };

    // Возвращаем значение
    return RGBPixel {
        red: (r_prime + m).round() as u8,
        green: (g_prime + m).round() as u8,
        blue: (b_prime + m).round() as u8,
    };
}
