//! Умный отсекатель тусклого
//!
//! Имеет строгое отсечение ниже заданного уровня, а также плавную кривую по области

use crate::{
    color::types::HSVPixel,
    filters::{configs::BlackThresholdConfig, types::BlackThreshold, ColorFilter},
};

impl BlackThreshold {
    /// Количество просчитанных отрезков на отрезке 0-255
    /// При таком разбиении человеческий глаз практически не видит разницы в
    /// изменении яркости (считается, то изменения на 1-3 пункта не являются
    /// заметными при Saturation [20%; 80%])
    pub const SAMPLES_AMOUNT: usize = 255 * 2;

    /// Конструктор
    ///
    /// Предрасчитывает значения для яркости, а также коэффициент для
    /// насыщенности, чтобы убрать вычисления из горячего цикла
    pub fn new(config: BlackThresholdConfig) -> Self {
        // Создаём буферы
        let mut precalculated_values = [0.0 as f32; Self::SAMPLES_AMOUNT];
        let mut precalculated_saturations_k = [0.0 as f32; Self::SAMPLES_AMOUNT];

        // Переводим из процентов
        let threshold = config.threshold * 255.0 / 100.0;
        let fade_range = config.fade_range * 255.0 / 100.0;

        // Просчитываем значения с указанной частотой
        for i in 0..Self::SAMPLES_AMOUNT {
            let value: f32 = i as f32 * 255.0 / (Self::SAMPLES_AMOUNT - 1) as f32;

            precalculated_values[i] =
                Self::calculate_value(value as f32, threshold, fade_range, config.falloff_exponent);
            precalculated_saturations_k[i] = Self::calculate_k(value as f32, threshold, fade_range);
        }

        Self {
            threshold: config.threshold,
            knee_upper_limit: config.threshold + config.fade_range,
            precalculated_values,
            precalculated_saturations_k,
        }
    }

    /// Расчёт коэффициента
    pub(self) fn calculate_k(value: f32, threshold: f32, fade_range: f32) -> f32 {
        if fade_range == 0.0 {
            return if value >= threshold { 1.0 } else { 0.0 };
        }
        let k = (value - threshold) / fade_range;
        k.clamp(0.0, 1.0)
    }

    /// Расчёт значений яркости
    pub(self) fn calculate_value(
        value: f32,
        threshold: f32,
        fade_range: f32,
        falloff_exponent: f32,
    ) -> f32 {
        value * (Self::calculate_k(value, threshold, fade_range)).powf(falloff_exponent)
    }

    /// Расчёт позиции в прерасчитанных значениях
    pub(self) fn calculate_position(value: f32) -> usize {
        let scale = (Self::SAMPLES_AMOUNT - 1) as f32 / 255.0;
        (value * scale) as usize
    }
}

impl ColorFilter for BlackThreshold {
    fn apply(&mut self, raw_colors: &mut crate::color::types::ColorBuffer) {
        for raw in AsMut::<[HSVPixel]>::as_mut(raw_colors).iter_mut() {
            // Если значение меньше минимального порога, отсекаем полностью
            if raw.value < self.threshold {
                raw.value = 0.0;
                raw.saturation = 0.0;
            }
            // Если значение входит в плавный диапазон, изменяем значения по
            // предрасчитанному диапазону
            else if raw.value < self.knee_upper_limit {
                let idx = Self::calculate_position(raw.value);
                raw.value = self.precalculated_values[idx];
                raw.saturation *= self.precalculated_saturations_k[idx];
            }
        }
    }
}
