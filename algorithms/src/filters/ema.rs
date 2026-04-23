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
        return Self {
            states: vec![RGBPixel::black(); amount],
        };
    }

    /// Конструктор с заданным хранилищем
    ///
    /// Создаёт фильтр, который принимает в себя готовое хранилище
    pub fn new_with_state(states: Vec<RGBPixel>) -> Self {
        return Self { states };
    }

    /// Пересчёт нового среднего
    ///
    /// **Поля:**
    /// - `raw_colors`: & mut [[RGBPixel]] - массив новых цветов для подсчёта
    pub fn process_ema(& mut self, raw_colors: & mut [RGBPixel]) {
        // Небольшая функция-помощник для правильного округления шага.
        // Она гарантирует, что шаг всегда будет минимум 1 (или -1),
        // пока разница не станет равна 0.
        fn calc_step(diff: i16) -> i16 {
            if diff > 0 {
                (diff + 3) / 4 // Округление "вверх" для положительных
            } else if diff < 0 {
                (diff - 3) / 4 // Округление "вниз" для отрицательных
            } else {
                0
            }
        }

        for (state, raw) in self.states.iter_mut().zip(raw_colors.iter_mut()) {
            state.red = (state.red as i16 + calc_step(raw.red as i16 - state.red as i16)) as u8;
            state.green =
                (state.green as i16 + calc_step(raw.green as i16 - state.green as i16)) as u8;
            state.blue = (state.blue as i16 + calc_step(raw.blue as i16 - state.blue as i16)) as u8;

            raw.red = state.red;
            raw.blue = state.blue;
            raw.green = state.green;
        }
    }
}
