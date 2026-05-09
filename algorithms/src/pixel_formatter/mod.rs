//! Модуль преобразования различных форматов хранения пикселя в единый для
//! алгоритмов формат из `color`
//!
//! Также реализуется итерация по массиву [u8] для удобства работы

use crate::color::types::RGBPixel;

pub mod abrg;
pub mod argb;
pub mod bgra;
pub mod bgrx;
pub mod rgba;
pub mod rgbx;
pub mod types;
pub mod xbgr;
pub mod xrgb;

/// Трейт преобразования различных форматов пикселя в [RGBPixel]
///
/// **Реализуемые методы:**
/// - `to_rgb` - преобразование к [RGBPixel]
pub trait PixelFormatter {
    /// Размер пикселя в байтах
    const SIZE: usize;

    /// Преобразование к [RGBPixel]
    ///
    /// Специальным образом (в зависимости от формата) преобразует данные в тип [RGBPixel]
    ///
    /// **Аргументы:**
    /// - `data`: &[[u8]] - указатель на кадр
    fn to_rgb(data: &[u8]) -> RGBPixel;
}

/// Итератор по пикселям
pub struct PixelIter<'a, F: PixelFormatter> {
    data: &'a [u8],
    step_bytes: usize,
    cursor: usize,
    _format: std::marker::PhantomData<F>,
}

impl<'a, F: PixelFormatter> PixelIter<'a, F> {
    pub fn new(data: &'a [u8], pixel_step: usize) -> Self {
        Self {
            data,
            // Шаг в байтах: (пропуск N пикселей + 1 текущий) * размер пикселя
            step_bytes: pixel_step * F::SIZE,
            cursor: 0,
            _format: std::marker::PhantomData,
        }
    }
}

/// Реализация итератора по пикселям
impl<'a, F: PixelFormatter> Iterator for PixelIter<'a, F> {
    type Item = RGBPixel;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor + F::SIZE <= self.data.len() {
            let pixel_data = &self.data[self.cursor..self.cursor + F::SIZE];
            let rgb = F::to_rgb(pixel_data);
            self.cursor += self.step_bytes;
            Some(rgb)
        } else {
            None
        }
    }
}
