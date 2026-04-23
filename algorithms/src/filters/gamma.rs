//! Алгоритм фильтрации значений с помощью гамма фильтра
//!
//! Данный алгоритм заранее рассчитывает гамма-таблицу по переданному
//! коэффициенту. Во время работы он преобразует значения согласно таблице. Это
//! необходимо в силу того, что человек различает яркость логорифмически

use crate::{color::types::RGBPixel, filters::types::GammaFilter};

/// Реализация методов [GammaFilter]
impl GammaFilter {
    /// Конструктор
    ///
    /// Предварительно рассчитывает гамма-таблицу по переданному коэффициенту
    ///
    /// **Поля:**
    /// - `gamma`: [f32] - гамма коэффициент
    pub fn new(gamma: f32) -> Self {
        let mut gamma_table = [0u8; 256];

        for i in 0..256 {
            // Нормализуем i до 0.0 - 1.0
            let v_in = i as f32 / 255.0;
            // Возводим в степень и масштабируем обратно до 0-255
            let v_out = v_in.powf(gamma) * 255.0;
            gamma_table[i] = v_out.round() as u8;
        }
        return Self { gamma_table };
    }

    /// Наложение гамма фильтра
    /// 
    /// **Поля:**
    /// - `raw_colors`: & mut [[RGBPixel]] - массив новых цветов для подсчёта
    pub fn apply_gamma(& mut self, raw_colors: & mut [RGBPixel]) {
        // Применяем фильтр по таблице
        for color in raw_colors.iter_mut() {
            color.red = self.gamma_table[color.red as usize];
            color.blue = self.gamma_table[color.blue as usize];
            color.green = self.gamma_table[color.green as usize];
        }
    }
}
