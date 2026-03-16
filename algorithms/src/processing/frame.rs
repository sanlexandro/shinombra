//! Алгоритм обхода кадра

use super::ChunkProcessor;
use crate::analytics::ColorAccumulator;
use crate::color::conversion::convert_hsv_to_rgb;
use crate::color::types::{HSVPixel, RGBPixel};
use crate::processing::types::{ColorEngine, Orientation};

/// KILLME
/// Ф-я вывода цвета в консоль для отладки
fn print_debug_color(rgb: RGBPixel) {
    // \x1b[48;2;R;G;Bm — цвет фона (TrueColor)
    // \x1b[0m — сброс
    println!(
        "\x1b[48;2;{};{};{}m      \x1b[0m Winner (RGB: {}, {}, {})",
        rgb.red, rgb.green, rgb.blue, rgb.red, rgb.green, rgb.blue
    );
}

/// Реализация методов ColorEngine
impl<P, A> ColorEngine<P, A>
where
    P: ChunkProcessor,
    A: ColorAccumulator,
{
    /// Конструктор
    ///
    /// Сохраняет конфигурацию
    ///
    /// **Аргументы:**
    /// - `processor`: [ChunkProcessor]    - метод обработки фрагмента
    /// - `accumulator`: [ColorAccumulator]- метод анализа цвета в фрагменте
    pub fn new(processor: P, accumulator: A) -> Self {
        return Self {
            processor: processor,
            accumulator: accumulator,
        };
    }

    /// Обработка кадра
    ///
    /// Данный метод вызывается для обработки кадра и использует предоставленные
    /// методы обработки фрагмента и анализа цвета
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]] - указатель на кадр (массив пикселей)
    pub fn process_frame(&mut self, byte_frame: &[u8]) -> HSVPixel {
        // Сбрасываем анализ
        self.accumulator.clear();

        // Обрабатываем фрагмент, используя предоставленный метод
        self.processor.process_chunk(
            byte_frame,
            0,
            Orientation::Horizontal,
            &mut self.accumulator,
        );

        // FIXME убери дебаг
        let hsv = self.accumulator.get_winner();

        print_debug_color(convert_hsv_to_rgb(hsv));

        return hsv;
    }
}
