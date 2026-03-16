//! Алгоритм построения гистограмм и обработки голосований

use super::types::{ColorBin, ColorHistogram};
use crate::color::types::HSVPixel;

/// Методы структуры ColorBin
impl ColorBin {
    /// Констркутор
    ///
    /// Создаёт сектор с 0-ым весом.
    pub fn new() -> Self {
        Self { weight: 0 }
    }

    /// Сброс информации о сегменте
    pub fn clear(&mut self) {
        self.weight = 0;
    }
}

/// Методы структуры ColorHistogram
impl ColorHistogram {
    /// Конструктор
    ///
    /// Создаёт массив сегментов со стандартными настройками
    pub fn new() -> Self {
        Self {
            // TODO: Считывание колличества сегментов из конфига
            bins: [ColorBin::new(); 37],
        }
    }

    /// Сброс (очистка) анализа
    pub fn clear(&mut self) {
        // Сбрасываем голоса о каждом сегменте
        for idx in 0..37 {
            // TODO: передавать кол-во сегментов через конфиг
            self.bins[idx].clear();
        }
    }

    /// Проведение голосования
    ///
    /// Распределяет голоса пикселей по секторам
    ///
    /// **Аргументы:**
    /// - `hsv_pixel`:[HSVPixel]   - голосующий HSV-пиксель
    pub fn process_vote(&mut self, hsv_pixel: HSVPixel) {
        // Вычисляем вес как произведение яркости и насыщенности
        let weight = (hsv_pixel.value * hsv_pixel.saturation) as u64;

        // Если вес слишком низкий, значит цвет близок к чёрному / белому
        if weight < 10 {
            // TODO: Считывание кол-ва сегментов из конфига
            self.bins[36].weight += weight + 1; // чтобы чёрный / белый всегда голосовал
            return;
        }

        // Определяем номер сектора
        let bin_idx = (hsv_pixel.hue / 10.0) as usize; // TODO: считывание размера сектора из конфига

        // Сохраняем голос
        self.bins[bin_idx].weight += weight;
    }
    /// Определение сектора-победителя
    ///
    /// Находит сектор с бОльшим колличеством
    ///
    /// **Аргументы:**
    /// - `bins`:&[[ColorBin]] - указатель на массив секторов
    ///
    /// **Выходные данные:**
    /// - `HSVPixel` - пиксель-победитель в HSV формате
    pub fn determining_winner(&mut self) -> HSVPixel {
        // Находим индекс сектора с максимальным весом
        let (winner_idx, winner_bin) = self
            .bins
            .iter()
            .enumerate()
            .max_by_key(|(_, bin)| bin.weight)
            .expect("bins must not be empty");

        // Если голосов вообще нет или победил "черный" (36-й индекс)
        if winner_idx == 36 || winner_bin.weight == 0 {
            return HSVPixel {
                hue: 0.0,
                saturation: 0.0,
                value: 0.0,
            };
        }

        // Рассчитываем Hue как центр сектора
        // Т.к. индекс 0 — это 0-10°, центр будет 5°
        let hue = (winner_idx as f32 * 10.0) + 5.0; // TODO: считывание размера сектора из конфига

        HSVPixel {
            hue,
            saturation: 1.0, // TODO: Усреднение цвета в сегменте
            value: 255.0,    // TODO: Усреднение цвета в сегменте (пока макс. яркость)
        }
    }
}
