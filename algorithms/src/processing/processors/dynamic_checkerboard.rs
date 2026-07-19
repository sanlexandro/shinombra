//! Алгоритм обхода фрагмента в динамическом шахматном порядке
//!
//! Данный алгоритм обходит фрагмент так же, как и жёсткий шахматный, но на
//! каждом фрагменте сдвигает эту жёсткую сетку на 1 шаг. Это позволяет
//! захватывать для анализа мелкие детали изображения

use crate::{
    analytics::ColorAnalyst, color::types::ColorBuffer, pixel_formatter::{PixelFormatter, PixelIter}, processing::{
        configs::ScreenConfig, processors::{
            ChunkProcessor, Orientation, configs::{ChunkTask, DynamicCheckerboardConfig}, types::{DynamicCheckerboardScanner, DynamicCheckerboardState},
        },
    },
};

/// Реализация методов DynamicCheckerboardConfig
impl DynamicCheckerboardConfig {
    /// Адаптация параметров под ориентацию фрагмента
    ///
    /// Возвращает кортеж: `(width, height, row_stride, pixel_step,
    /// column_crawl, row_crawl)`:[usize]
    pub fn get_params_for(
        &self,
        orientation: Orientation,
    ) -> (usize, usize, usize, usize, usize, usize) {
        match orientation {
            Orientation::Horizontal => (
                self.config.width.0,
                self.config.height.0,
                self.row_stride,
                self.pixel_step,
                self.column_crawl,
                self.row_crawl,
            ),
            Orientation::Vertical => (
                self.config.height.0,
                self.config.width.0,
                self.pixel_step,
                self.row_stride,
                self.row_crawl,
                self.column_crawl,
            ),
        }
    }
}

/// Реализация методов DynamicCheckerboardScanner
impl DynamicCheckerboardScanner {
    /// Конструктор
    ///
    /// **Поля:**
    /// - `alg_config`: [DynamicCheckerboardConfig] - конфигурация для шахматки
    /// - `screen_config`: [ScreenConfig]           - конфигурация экрана
    pub fn new(alg_config: DynamicCheckerboardConfig, screen_config: ScreenConfig) -> Self {
        return Self {
            alg_config,
            screen_config,
        };
    }
}

/// Реализация трейта для DynamicCheckerboardScanner
impl<Formatter: PixelFormatter, Analyst: ColorAnalyst> ChunkProcessor<Formatter, Analyst>
    for DynamicCheckerboardScanner
where
    // Waiting for RFC 2089
    ColorBuffer: From<Vec<Analyst::OutputFormat>>,
    Vec<Analyst::OutputFormat>: From<ColorBuffer>,
    ColorBuffer: AsMut<[Analyst::OutputFormat]>,
{
    type State = DynamicCheckerboardState;

    /// Обработка фрагмента (динамический шахматный порядок)
    ///
    /// Данный метод обходит фрагмент в шахматном порядке, сохраняя данные для
    /// анализа в предоставленный метод обработки цвета. На каждом следующем
    /// кадре сетка смещается на заданную величину
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]]                          - указатель на кадр
    ///   (массив пикселей)
    /// - `chunk_task`: &[ChunkTask]                     - "задание" фрагмента
    /// - `chunk_state`: &mut [DynamicCheckerboardState] - пустое состояние
    ///   фрагмента
    /// - `&mut analyst`: [ColorAnalyst]                 - указатель на
    ///   структуру (метод) накопления данных для дальнейшего определения
    ///   результирующего цвета
    fn process_chunk(
        &self,
        byte_frame: &[u8],
        chunk_task: &ChunkTask,
        chunk_state: &mut Self::State,
        analyst: &mut Analyst,
    ) {
        // Определяем реальный размер строки в байтах
        let row_width = self.screen_config.frame_width_px.as_bytes(Formatter::SIZE);

        // В зависимости от положения кадра определяем его ширину и высоту
        let (chunk_width, chunk_height, row_stride, pixel_step, column_crawl, row_crawl) =
            self.alg_config.get_params_for(chunk_task.orientation);

        // Увеличиваем текущее смещение
        chunk_state.current_column += column_crawl;
        chunk_state.current_row += row_crawl;

        // Проверяем границы: если выходит за границы, сбрасываем с учётом остатка
        if chunk_state.current_column >= pixel_step {
            chunk_state.current_column %= pixel_step;
        }
        if chunk_state.current_row >= row_stride {
            chunk_state.current_row %= row_stride;
        }

        // Цикл построчного чтения (читаем каждую `row_stride` строку смещаясь
        // на текущее смещение по строке)
        for row_idx in (chunk_state.current_row..chunk_height).step_by(row_stride) {
            // Вычисляем стартовый индекс строки (стартовый индекс + (ширина экрана
            // row_stride кол-во строк))
            let row_start_index = chunk_task.start_index + row_width * row_idx;

            // В зависимости от чётности строки начинаем строку либо с самого начала
            // либо со сдвигом на половину шага чтения строки
            //
            // Важно!! Сюда прибавляем текущий сдвиг по колонке!
            let start = if (row_idx / row_stride) % 2 == 1 {
                row_start_index + (pixel_step / 2) * Formatter::SIZE
            } else {
                row_start_index
            } + chunk_state.current_column * Formatter::SIZE;
            let end = row_start_index + chunk_width * Formatter::SIZE;

            // Делаем срез строки (от стартового индекса строки, до (него + ширина
            // фрагмента))
            let row_bytes = &byte_frame[start..end];
            let pixels = PixelIter::<Formatter>::new(row_bytes, pixel_step);

            // Итерируемся по срезу (читаем каждый `pixel_step` пиксель)
            for rgb in pixels {
                // Проводим голосование
                analyst.add_data(rgb);
            }
        }
    }
}
