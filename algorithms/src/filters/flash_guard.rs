//! Защита от вспышек
//!
//! Использует EMA для сглаживания изменений превышающих максимальный заданный
//! порог

use crate::{
    color::types::HSVPixel,
    filters::{configs::FlashGuardConfig, types::FlashGuard, ColorFilter},
};

impl FlashGuard {
    /// Количество просчитанных отрезков на отрезке 0-255
    /// При таком разбиении человеческий глаз практически не видит разницы в
    /// изменении яркости (считается, то изменения на 1-3 пункта не являются
    /// заметными при Saturation [20%; 80%])
    pub const SAMPLES_AMOUNT: usize = 255 * 2;

    /// Конструктор
    ///
    /// предрассчитывает слагаемые в зависимости от разности
    pub fn new(config: FlashGuardConfig, led_amount: usize) -> Self {
        // Переводим из процентов
        let sensitivity: f32 = config.sensitivity * 255.0 / 100.0;

        let mut terms = [0.0 as f32; Self::SAMPLES_AMOUNT];

        // Рассчитываем добавочные коэффициенты в зависимости от дельты
        for i in 0..Self::SAMPLES_AMOUNT {
            let difference: f32 = i as f32 * 255.0 / (Self::SAMPLES_AMOUNT - 1) as f32;

            terms[i] = Self::calculate_alpha(config.alpha, difference, sensitivity) * difference;
        }

        Self {
            terms,
            previous_values: vec![0.0 as f32; led_amount],
        }
    }

    /// Расчёт динамического alpha коэффициента
    pub(super) fn calculate_alpha(base_alpha: f32, difference: f32, sensitivity: f32) -> f32 {
        base_alpha / (1.0 + (difference / sensitivity).powi(2))
    }

    /// Расчёт позиции в прерассчитанных значениях
    pub(self) fn calculate_position(difference: f32) -> usize {
        let scale = (Self::SAMPLES_AMOUNT - 1) as f32 / 255.0;
        (difference * scale) as usize
    }
}

impl ColorFilter for FlashGuard {
    fn apply(&mut self, raw_colors: &mut crate::color::types::ColorBuffer) {
        for (i, raw) in AsMut::<[HSVPixel]>::as_mut(raw_colors)
            .iter_mut()
            .enumerate()
        {
            // Вычисляем разницу
            let difference = raw.value - self.previous_values[i];
            let abs_difference = difference.abs();

            // Определяем слагаемое
            let summand = self.terms[Self::calculate_position(abs_difference)];

            // Если difference была отрицательной — мы вычитаем, если положительной — прибавляем
            let new_value = if difference >= 0.0 {
                self.previous_values[i] + summand
            } else {
                self.previous_values[i] - summand
            };

            self.previous_values[i] = new_value;
            raw.value = new_value;
        }
    }
}
