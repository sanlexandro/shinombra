//! Алгоритмы для преобразования форматов

use std::unreachable;

use crate::color::types::ColorBuffer;

use super::types::{HSVPixel, RGBPixel};

/// Преобразование RGB формата в HSV
impl From<RGBPixel> for HSVPixel {
    fn from(rgb: RGBPixel) -> Self {
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
        HSVPixel {
            hue,
            saturation,
            value,
        }
    }
}

/// Преобразование HSV формата в RGB
impl From<HSVPixel> for RGBPixel {
    fn from(hsv: HSVPixel) -> Self {
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
        RGBPixel {
            red: (r_prime + m).round() as u8,
            green: (g_prime + m).round() as u8,
            blue: (b_prime + m).round() as u8,
        }
    }
}

/// Преобразование [Vec<RGBPixel>] в [ColorBuffer]
impl From<Vec<RGBPixel>> for ColorBuffer {
    fn from(value: Vec<RGBPixel>) -> Self {
        ColorBuffer::RGB(value)
    }
}

/// Преобразование [Vec<HSVPixel>] в [ColorBuffer]
impl From<Vec<HSVPixel>> for ColorBuffer {
    fn from(value: Vec<HSVPixel>) -> Self {
        ColorBuffer::HSV(value)
    }
}

/// Преобразование [ColorBuffer] в [Vec<RGBPixel>]
///
/// Поддерживает ленивое преобразование типов
impl From<ColorBuffer> for Vec<RGBPixel> {
    fn from(value: ColorBuffer) -> Self {
        match value {
            ColorBuffer::RGB(vec) => vec,
            ColorBuffer::HSV(vec) => vec.into_iter().map(Into::into).collect(),
        }
    }
}

/// Преобразование [ColorBuffer] в [Vec<HSVPixel>]
///
/// Поддерживает ленивое преобразование типов
impl From<ColorBuffer> for Vec<HSVPixel> {
    fn from(value: ColorBuffer) -> Self {
        match value {
            ColorBuffer::HSV(vec) => vec,
            ColorBuffer::RGB(vec) => vec.into_iter().map(Into::into).collect(),
        }
    }
}

/// Вынимание среза [[RGBPixel]] из [ColorBuffer]
///
/// Поддерживает ленивое преобразование типов
///
/// При добавлении нового формата ОБЯЗАТЕЛЬНО добавить ленивое преобразование
impl AsMut<[RGBPixel]> for ColorBuffer {
    fn as_mut(&mut self) -> &mut [RGBPixel] {
        if let ColorBuffer::HSV(vec) = self {
            *self = ColorBuffer::RGB(vec.iter().map(|&p| p.into()).collect());
        }

        match self {
            ColorBuffer::RGB(vec) => vec.as_mut_slice(),
            _ => unreachable!(),
        }
    }
}

/// Вынимание среза [[HSVPixel]] из [ColorBuffer]
///
/// Поддерживает ленивое преобразование типов
///
/// При добавлении нового формата ОБЯЗАТЕЛЬНО добавить ленивое преобразование
impl AsMut<[HSVPixel]> for ColorBuffer {
    fn as_mut(&mut self) -> &mut [HSVPixel] {
        if let ColorBuffer::RGB(vec) = self {
            *self = ColorBuffer::HSV(vec.iter().map(|&p| p.into()).collect());
        }

        match self {
            ColorBuffer::HSV(vec) => vec.as_mut_slice(),
            _ => unreachable!(),
        }
    }
}

impl From<&ColorBuffer> for Vec<RGBPixel> {
    fn from(value: &ColorBuffer) -> Self {
        match value {
            ColorBuffer::RGB(vec) => vec.clone(),
            ColorBuffer::HSV(vec) => vec.iter().map(|&pixel| pixel.into()).collect(),
        }
    }
}
