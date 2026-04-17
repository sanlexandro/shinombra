//! Алгоритм построения гистограмм и обработки голосований

use super::types::{ColorBin, ColorHistogram};
use crate::color::types::HSVPixel;

/// Методы структуры ColorBin
impl ColorBin {
    /// Конструктор
    ///
    /// Создаёт сектор с 0-ым весом.
    pub fn new() -> Self {
        Self {
            weight: 0,
            votes: 0,
            sum_pixel: HSVPixel::black(),
        }
    }

    /// Сброс информации о сегменте
    pub fn clear(&mut self) {
        self.weight = 0;
        self.votes = 0;
        self.sum_pixel = HSVPixel::black();
    }
}

/// Методы структуры ColorHistogram
impl ColorHistogram {
    /// Конструктор
    ///
    /// Создаёт массив сегментов со стандартными настройками
    pub fn new() -> Self {
        Self {
            // TODO: Считывание количества сегментов из конфига
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
        let color_weight = (hsv_pixel.value * hsv_pixel.saturation * 100.0) as u64;

        // Если насыщенность слишком низкая,
        // пиксель идет в «бесцветный» сектор (36)
        if hsv_pixel.saturation < 0.15 {
            // TODO: получение кол-ва сегментов из конфига
            let bin = &mut self.bins[36];

            // Вес для белого тем выше, чем выше яркость
            bin.weight += (hsv_pixel.value * 100.0) as u64;
            bin.votes += 1;

            // Накапливаем значения (hue тут не важен, но для единообразия пишем всё)
            bin.sum_pixel.hue += hsv_pixel.hue;
            bin.sum_pixel.saturation += hsv_pixel.saturation;
            bin.sum_pixel.value += hsv_pixel.value;
            return;
        }

        // Если пиксель цветной, определяем его сектор
        let bin_idx = (hsv_pixel.hue / 10.0) as usize;
        if bin_idx >= 36 {
            return;
        } // Защита от выхода за границы (360.0 / 10.0)

        let result_bin = &mut self.bins[bin_idx];

        result_bin.weight += color_weight;
        result_bin.votes += 1;

        result_bin.sum_pixel.hue += hsv_pixel.hue;
        result_bin.sum_pixel.saturation += hsv_pixel.saturation;
        result_bin.sum_pixel.value += hsv_pixel.value;
    }

    /// Определение сектора-победителя
    ///
    /// Находит сектор с бОльшим количеством
    ///
    /// **Аргументы:**
    /// - `bins`:&[[ColorBin]] - указатель на массив секторов
    ///
    /// **Выходные данные:**
    /// - `HSVPixel` - пиксель-победитель в HSV формате
    pub fn determining_winner(&mut self) -> HSVPixel {
        // Ищем сектор с максимальным весом
        let (winner_idx, winner_bin) = self
            .bins
            .iter()
            .enumerate()
            .max_by_key(|(_, bin)| bin.weight)
            .expect("bins must not be empty");

        // Если веса вообще нет — выключаем ленту
        if winner_bin.weight == 0 {
            return HSVPixel::black();
        }

        // Вычисляем среднее значение
        let votes = winner_bin.votes as f32;
        let avg_hue = winner_bin.sum_pixel.hue / votes;
        let avg_sat = winner_bin.sum_pixel.saturation / votes;
        let avg_val = winner_bin.sum_pixel.value / votes;

        // Если победил сектор 36 (ахроматический)
        if winner_idx == 36 {
            // Если яркость совсем низкая — возвращаем черный
            if avg_val < 0.05 {
                return HSVPixel::black();
            }
            // Если яркость есть — это белый/серый (saturation 0)
            return HSVPixel {
                hue: 0.0,
                saturation: 0.0,
                value: avg_val,
            };
        }

        // Для обычных секторов возвращаем честное усредненное значение
        HSVPixel {
            hue: avg_hue,
            saturation: avg_sat,
            value: avg_val,
        }
    }

    /// Доступ к сегментам (только для тестов)
    #[cfg(test)]
    pub fn get_bins(&self) -> &[ColorBin; 37] {
        &self.bins
    }
}
