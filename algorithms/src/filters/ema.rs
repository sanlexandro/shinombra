//! Алгоритм фильтрации значения с помощью экспоненциального скользящего среднего
//!
//! Данный алгоритм смешивает предыдущее значение с вычисленным в пропорции 3:1,
//! в результате чего достигается плавность изменения цвета между кадрами

use crate::{
    color::{
        types::{ColorBuffer, RGBPixel},
        Color,
    },
    filters::{configs::EmaFilterConfig, types::EmaFilter, ColorFilter},
};

impl EmaFilter {
    /// Конструктор
    ///
    /// Создаёт хранилище с чёрным цветом
    pub fn new(config: EmaFilterConfig) -> Self {
        let alpha = (config.alpha * 256.0) as i32;
        let inverted_alpha = 256 - alpha;

        return Self {
            states: vec![RGBPixel::black(); config.amount],
            alpha,
            inverted_alpha,
        };
    }
}

impl ColorFilter for EmaFilter {
    /// Пересчёт нового среднего
    fn apply(&mut self, raw_colors: &mut ColorBuffer) {
        for (state, raw) in self
            .states
            .iter_mut()
            .zip(AsMut::<[RGBPixel]>::as_mut(raw_colors).iter_mut())
        {
            // Формула: NewState = (Alpha * Raw + (1 - Alpha) * OldState) / 256
            // Используем i32, чтобы избежать переполнения при умножении

            state.red =
                ((self.alpha * raw.red as i32 + self.inverted_alpha * state.red as i32 + 128) >> 8)
                    as u8;
            state.green =
                ((self.alpha * raw.green as i32 + self.inverted_alpha * state.green as i32 + 128)
                    >> 8) as u8;
            state.blue =
                ((self.alpha * raw.blue as i32 + self.inverted_alpha * state.blue as i32 + 128)
                    >> 8) as u8;

            *raw = *state;
        }
    }
}
