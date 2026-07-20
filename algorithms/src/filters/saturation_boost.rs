//! Усилитель насыщенности
//!
//! Т.к. подсветка светит на стену за монитором, а также может иметь матовый
//! рассеиватель, насыщенность цветов падает. Для этого необходим усилитель
//! насыщенности.
//!
//! Данный вариант реализует умную сигмоиду (S-curve) с отсечением, которая
//! позволяет не трогать цвета до определённого порога, а также плавно изменять
//! выбранный диапазон.

use crate::{
    color::types::HSVPixel,
    filters::{configs::SaturationBoostConfig, types::SaturationBoost, ColorFilter},
};

impl SaturationBoost {
    /// Частота дискретизации для LUT (0.001)
    pub const SAMPLES_AMOUNT: usize = 1000;

    /// Конструктор
    ///
    /// предрассчитывает значения насыщенности в LUT с заданной частотой дискретизации
    pub fn new(config: SaturationBoostConfig) -> Self {
        // Выделяем место под LUT
        let mut saturation_table = [0.0 as f32; Self::SAMPLES_AMOUNT];

        // Вычисляем с заданной частотой дискретизации
        for i in 0..Self::SAMPLES_AMOUNT {
            let saturation: f32 = i as f32 / (Self::SAMPLES_AMOUNT - 1) as f32;

            saturation_table[i] = Self::calculate_saturation(saturation, &config);
        }

        Self { saturation_table }
    }

    /// Вычисление насыщенности
    pub(self) fn calculate_saturation(saturation: f32, config: &SaturationBoostConfig) -> f32 {
        // Переводим из %
        let floor = config.floor / 100.0;

        // Нормализуем
        let s_norm = (saturation - floor) / (1.0 - floor);

        // Считаем S-curve
        s_norm.clamp(0.0, 1.0).powf(1.0 / config.boost_exponent)
    }

    /// Вычисление позиции в LUT
    pub(self) fn calculate_position(saturation: f32) -> usize {
        (saturation * (Self::SAMPLES_AMOUNT - 1) as f32) as usize
    }
}

impl ColorFilter for SaturationBoost {
    fn apply(&mut self, raw_colors: &mut crate::color::types::ColorBuffer) {
        // Просто смотрим в LUT
        for raw in AsMut::<[HSVPixel]>::as_mut(raw_colors).iter_mut() {
            raw.saturation = self.saturation_table[Self::calculate_position(raw.saturation)];
        }
    }
}
