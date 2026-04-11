//! Алгоритм фильтрации значения с помощью экспоненциального скользящего среднего
//! 
//! Данный алгоритм смешивает предыдущее значение с вычисленным в пропорции 3:1,
//! в результате чего достигается плавность изменения цвета между кадрами

use crate::{color::types::RGBPixel, filters::types::EmaFilter};

impl EmaFilter {
    /// Конструктор
    /// 
    /// Создаёт хранилище с чёрным цветом
    pub fn new(amount: usize) -> Self {
        return Self { states: vec![RGBPixel::black(); amount] };
    }

    /// Пересчёт нового среднего
    /// 
    /// **Поля:**
    /// - `raw_colors`: &[[RGBPixel]] - массив новых цветов для подсчёта
    /// 
    /// **Выходные данные:**
    /// - ?[RGBPixel] - массив фильтрованных цветов в формате RGB
    pub fn process_ema (&mut self, raw_colors: &[RGBPixel]) -> &[RGBPixel] {
        // Просто вычисляем EVA для каждого пикселя
        for (state, raw) in self.states.iter_mut().zip(raw_colors.iter()) {
            state.red = ((state.red as u16 * 3 + raw.red as u16) >> 2) as u8;
            state.green = ((state.green as u16 * 3 + raw.green as u16) >> 2) as u8;
            state.blue = ((state.blue as u16 * 3 + raw.blue as u16) >> 2) as u8;
        }
        return &self.states;
    }
}