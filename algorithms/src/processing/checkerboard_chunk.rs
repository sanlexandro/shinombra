//! Алгоритм обработки фрагмента в жёстком шахматном порядке
//! 
//! Данный алгоритм обходит фрагмет в "шахматном порядке", пропуская большие
//! области фрагмента. Он построен на предположении "на экране нет достаточно
//! маленьких объектов, которые могут проскачить между сеткой". Шахматный
//! алгоритм выигрывает по скорости другие, т.е. не требует больших вычислений,
//! но может проигрывать в качестве, если размер сетки был выбран неправильно

use super::types::{CheckerboardConfig, CheckerboardScanner, Orientation};
use crate::analytics::ColorAccumulator;
use crate::color::conversion::convert_rgb_to_hsv;
use crate::color::types::RGBPixel;
use crate::processing::types::ScreenConfig;

/// Реализация методов CheckerboardConfig
impl CheckerboardConfig {
    /// Адаптация параметров под ориентацию фрагмента
    ///
    /// Возвращает кортеж: `(width, height, row_stride, pixel_step)`:[usize]
    pub fn get_params_for(&self, orientation: Orientation) -> (usize, usize, usize, usize) {
        match orientation {
            Orientation::Horizontal => (
                self.chunk_width,
                self.chunk_height,
                self.row_stride,
                self.pixel_step,
            ),
            Orientation::Vertical => (
                self.chunk_height,
                self.chunk_width,
                self.pixel_step,
                self.row_stride,
            ),
        }
    }
}

/// Реализация методов CheckerboardScanner
impl CheckerboardScanner {
    /// Конструктор
    ///
    /// TODO: дописать комментарий
    pub fn new(alg_config: CheckerboardConfig, screen_config: ScreenConfig) -> Self {
        return Self {
            alg_config,
            screen_config,
        };
    }

    /// Обработка фрагмента (жёсткий шахматный порядок)
    ///
    /// Данный метод обходит фрагмент в шахматном порядке, сохраняя данные для
    /// анализа в предоставленный метод обработки цвета
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]]                   - указатель на кадр (массив
    ///   пикселей)
    /// - `chunk_start_index`: [usize]            - стартовый индекс пикселя, с
    ///   которого начинается фрагмент (верхний левый угол)
    /// - `chunk_orientation`: [Orientation]     - ориентация фрагмента
    /// - `accumulator`: &mut [ColorAccumulator] - метод анализа цвета
    pub fn process_checkerboard_chunk<A: ColorAccumulator>(
        &self,
        byte_frame: &[u8],
        chunk_start_index: usize,
        chunk_orientation: Orientation,
        accumulator: &mut A,
    ) {
        // Определяем реальный размер строки в байтах
        let row_width = self.screen_config.frame_width * 4;

        // В зависимости от положения кадра определяем его ширину и высоту
        let (chunk_width, chunk_height, row_stride, pixel_step) =
            self.alg_config.get_params_for(chunk_orientation);

        // Цикл построчного чтения (читаем каждую `row_stride` строку)
        for row_idx in (0..chunk_height).step_by(row_stride) {
            // Вычисляем стартовый индекс строки (стартовый индекс + (ширина экрана
            // row_strideкол-во строк))
            let row_start_index = chunk_start_index + row_width * row_idx;

            // В зависимости от чётности строки начинаем строку либо с самого начала
            // либо со сдвигом на половину шага чтения строки
            let start = if (row_idx / row_stride) % 2 == 1 {
                row_start_index + (pixel_step / 2) * 4
            } else {
                row_start_index
            };
            let end = row_start_index + chunk_width * 4;

            // fixme добавить логику смещения строк по чётности

            // Делаем срез строки (от стартового индекса строки, до (него + ширина
            // фрагмента))
            let row_bytes = &byte_frame[start..end];
            let pixels = row_bytes.chunks_exact(4).step_by(pixel_step);

            // Итерируемся по срезу (читаем каждый `pixel_step` пиксель)
            for bgr in pixels {
                let rgb = RGBPixel {
                    red: bgr[2],
                    green: bgr[1],
                    blue: bgr[0],
                };

                let hsv = convert_rgb_to_hsv(rgb);

                // Проводим голосование
                accumulator.add_data(hsv);
            }
        }
    }
}
