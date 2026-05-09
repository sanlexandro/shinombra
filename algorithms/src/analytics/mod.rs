//! Модуль определения результирующего цвета
//!
//! Данный модуль реализует алгоритмы, которые собирают пул данных, основанный
//! на переданных для обработки пикселях. После чего собранные данные
//! анализируются и вычисляется результирующий цвет
//!
//! **Методы:**
//! - Гистограммы [types::ColorHistogram] - метод, основанный на разбиение HSV "круга" на отдельные
//!   сектора. Каждый переданный пиксель голосует за сектор, в котором
//!   находится, в результате чего определяется сектор с большим количеством
//!   голосов

pub mod configs;
pub mod histogram;
pub mod registry;
pub mod types;

use crate::color::types::RGBPixel;

/// Трейт анализа цвета
///
/// **Методы:**
/// - `add_data`   - добавление данных для анализа
/// - `get_winner` - вычисление результата анализа
pub trait ColorAnalyst: Clone {
    /// Сброс (очистка) анализа
    fn clear(&mut self);

    /// Добавление пикселя в текущую модель анализа
    ///
    /// Этот метод вызывается для каждого выбранного пикселя в чанке
    ///
    /// **Аргументы:**
    /// - `rgb: RGBPixel` - пиксель в RGB формате
    fn add_data(&mut self, rgb: RGBPixel);

    /// Вычисление итогового "победившего" цвета на основе накопленных данных
    ///
    /// **Выходные данные:**
    ///  - `RGBPixel` - победивший цвет в RGB формате
    fn get_winner(&mut self) -> RGBPixel;
}

// Реализация трейта для ColorHistogram
impl ColorAnalyst for types::ColorHistogram {
    fn clear(&mut self) {
        self.clear();
    }
    fn add_data(&mut self, rgb: RGBPixel) {
        self.process_vote(rgb);
    }
    fn get_winner(&mut self) -> RGBPixel {
        self.determining_winner()
    }
}
