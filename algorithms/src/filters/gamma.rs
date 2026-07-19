//! Алгоритм фильтрации значений с помощью гамма фильтра
//!
//! Данный алгоритм заранее рассчитывает гамма-таблицу по переданному
//! коэффициенту. Во время работы он преобразует значения согласно таблице. Это
//! необходимо в силу того, что человек различает яркость логорифмически

use crate::{
    color::types::{ColorBuffer, RGBPixel},
    filters::{configs::GammaConfig, types::Gamma, ColorFilter},
};

impl Gamma {
    /// Конструктор
    ///
    /// Предварительно рассчитывает гамма-таблицу по переданному коэффициенту
    ///
    /// **Поля:**
    /// - `config`: [GammaConfig] - конфигурация для gamma-фильтра
    pub fn new(config: GammaConfig) -> Self {
        let mut gamma_table = [0u8; 256];

        for i in 0..256 {
            // Нормализуем i до 0.0 - 1.0
            let v_in = i as f32 / 255.0;
            // Возводим в степень и масштабируем обратно до 0-255
            let v_out = v_in.powf(config.gamma) * 255.0;
            gamma_table[i] = v_out.round() as u8;
        }
        return Self { gamma_table };
    }
}

impl ColorFilter for Gamma {
    /// Наложение гамма фильтра
    fn apply(&mut self, raw_colors: &mut ColorBuffer) {
        // Применяем фильтр по таблице
        for color in AsMut::<[RGBPixel]>::as_mut(raw_colors).iter_mut() {
            color.red = self.gamma_table[color.red as usize];
            color.blue = self.gamma_table[color.blue as usize];
            color.green = self.gamma_table[color.green as usize];
        }
    }
}
