//! Медианный фильтр
//!
//! Реализация на окно, размером 3

use std::vec;

use crate::{
    color::{types::RGBPixel, Color},
    filters::{types::Median, ColorFilter},
};

impl Median {
    /// Конструктор
    pub fn new(led_amount: usize) -> Self {
        Self {
            previous: [
                vec![RGBPixel::black(); led_amount],
                vec![RGBPixel::black(); led_amount],
            ],
        }
    }

    /// Нахождение медианы
    #[inline(always)]
    pub(super) fn median(a: u8, b: u8, c: u8) -> u8 {
        a.min(b).max(a.max(b).min(c))
    }
}

impl ColorFilter for Median {
    fn apply(&mut self, raw_colors: &mut crate::color::types::ColorBuffer) {
        for (i, raw) in AsMut::<[RGBPixel]>::as_mut(raw_colors)
            .iter_mut()
            .enumerate()
        {
            // Сохраняем в буфер
            let buf = *raw;

            let rgb_0 = self.previous[0][i];
            let rgb_1 = self.previous[1][i];

            // Находим медиану
            raw.red = Self::median(rgb_0.red, rgb_1.red, raw.red);
            raw.green = Self::median(rgb_0.green, rgb_1.green, raw.green);
            raw.blue = Self::median(rgb_0.blue, rgb_1.blue, raw.blue);

            // Сдвигаем буфер
            self.previous[0][i] = rgb_1;
            self.previous[1][i] = buf;
        }
    }
}
