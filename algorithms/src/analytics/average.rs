//! Алгоритм анализа цвета через простое среднее арифметическое

use crate::{
    analytics::{types::Average, ColorAnalyst},
    color::types::RGBPixel,
};

impl Average {
    /// Конструктор
    ///
    /// Создаёт хранилище среднего арифметического с нулевыми значениями
    pub fn new() -> Average {
        Self {
            sum_red: 0,
            sum_green: 0,
            sum_blue: 0,
            amount: 0,
        }
    }
}

impl ColorAnalyst for Average {
    /// Выходной формат в [RGBPixel]
    type OutputFormat = RGBPixel;

    /// Сброс (очистка) анализа
    ///
    /// Не очищает, а создаёт новую пустую реализацию, т.к. это производительнее
    fn clear(&mut self) {
        *self = Self::new();
    }

    /// Добавить данные для обработки
    ///
    /// Добавляет значения из пикселя в сумму и увеличивает счётчик
    ///
    /// **Аргументы:**
    /// - `rgb`: [RGBPixel] - пиксель в RGB формате
    fn add_data(&mut self, rgb: RGBPixel) {
        self.sum_red += rgb.red as u32;
        self.sum_green += rgb.green as u32;
        self.sum_blue += rgb.blue as u32;
        self.amount += 1;
    }

    /// Определение победителя
    ///
    /// Вычисляет среднее арифметическое для текущего анализа
    ///
    /// **Выходные данные:**
    ///  - [RGBPixel] - победивший цвет в RGB формате
    fn get_winner(&mut self) -> RGBPixel {
        let amt = self.amount as u32;

        RGBPixel {
            red: (self.sum_red / amt) as u8,
            green: (self.sum_green / amt) as u8,
            blue: (self.sum_blue / amt) as u8,
        }
    }
}
