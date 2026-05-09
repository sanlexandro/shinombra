//! Алгоритм построения гистограмм и обработки голосований

use super::types::{ColorBin, ColorHistogram};
use crate::{
    analytics::{configs::ColorHistogramConfig, types::MAX_BINS},
    color::{
        conversion::{convert_hsv_to_rgb, convert_rgb_to_hsv},
        types::{HSVPixel, RGBPixel},
    },
};

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
    /// Создаёт массив сегментов со стандартными настройками, также рассчитывает
    /// количество рабочих ячеек и коэффициент для быстрых вычислений корзины
    /// для текущего цвета
    pub fn new(config: ColorHistogramConfig) -> Self {
        let active_amount = config.precision_level * 6 + 1;
        let hue_to_bin_scale = (active_amount - 1) as f32 / 360.0;

        Self {
            bins: [ColorBin::new(); MAX_BINS + 1],
            active_amount,
            hue_to_bin_scale,
        }
    }

    /// Сброс (очистка) анализа
    pub fn clear(&mut self) {
        // Сбрасываем голоса о каждом сегменте
        for idx in 0..self.active_amount {
            self.bins[idx].clear();
        }
    }

    /// Проведение голосования
    ///
    /// Распределяет голоса пикселей по секторам
    ///
    /// **Аргументы:**
    /// - `hsv_pixel`:[RGBPixel]   - голосующий HSV-пиксель
    pub fn process_vote(&mut self, rgb_pixel: RGBPixel) {
        // Преобразуем
        let hsv_pixel = convert_rgb_to_hsv(rgb_pixel);

        // Вычисляем вес как произведение яркости и насыщенности
        let color_weight = (hsv_pixel.value * hsv_pixel.saturation * 100.0) as u64;

        // Если насыщенность слишком низкая,
        // пиксель идет в «бесцветный» сектор (последний)
        if hsv_pixel.saturation < 0.15 {
            let bin = &mut self.bins[self.active_amount - 1];

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
        let bin_idx = (hsv_pixel.hue * self.hue_to_bin_scale) as usize;
        let bin_idx = bin_idx.min(self.active_amount - 2); // Защита от выхода за границы

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
    pub fn determining_winner(&mut self) -> RGBPixel {
        // Ищем сектор с максимальным весом
        let (winner_idx, winner_bin) = self.bins[..self.active_amount] // Итерируемся только по активным!
            .iter()
            .enumerate()
            .max_by_key(|(_, bin)| bin.weight)
            .expect("bins must not be empty");

        // Если веса вообще нет — выключаем ленту
        if winner_bin.weight == 0 {
            return RGBPixel::black();
        }

        // Вычисляем среднее значение
        let votes = winner_bin.votes as f32;
        let avg_hue = winner_bin.sum_pixel.hue / votes;
        let avg_sat = winner_bin.sum_pixel.saturation / votes;
        let avg_val = winner_bin.sum_pixel.value / votes;

        // Если победил ахроматический сектор
        if winner_idx == self.active_amount - 1 {
            // Если яркость совсем низкая — возвращаем черный
            if avg_val < 0.05 {
                return RGBPixel::black();
            }
            // Если яркость есть — это белый/серый (saturation 0)
            return convert_hsv_to_rgb(HSVPixel {
                hue: 0.0,
                saturation: 0.0,
                value: avg_val,
            });
        }

        // Для обычных секторов возвращаем честное усредненное значение
        convert_hsv_to_rgb(HSVPixel {
            hue: avg_hue,
            saturation: avg_sat,
            value: avg_val,
        })
    }

    /// Доступ к сегментам (только для тестов)
    #[cfg(test)]
    pub fn get_bins(&self) -> &[ColorBin; MAX_BINS + 1] {
        &self.bins
    }
}
