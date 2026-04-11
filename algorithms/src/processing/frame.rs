//! Алгоритм обхода кадра

use super::configs::*;
use super::measures::calculate_mm_to_px_k;
use super::types::*;
use super::ChunkProcessor;
use crate::analytics::ColorAnalyst;
use crate::color::conversion::convert_hsv_to_rgb;
use crate::color::types::RGBPixel;
use crate::filters::ColorFilter;
use crate::processing::measures::calculate_px_x_y_to_bytes;
use crate::units::*;

/// Реализация методов ColorEngine
impl<Processor, Analyst, Filter> ColorEngine<Processor, Analyst, Filter>
where
    Processor: ChunkProcessor,
    Analyst: ColorAnalyst,
    Filter: ColorFilter
{
    /// Конструктор
    ///
    /// Сохраняет конфигурацию
    ///
    /// **Аргументы:**
    /// - `processor`: [ChunkProcessor]    - метод обработки фрагмента
    /// - `analyst`: [ColorAnalyst]- метод анализа цвета в фрагменте
    pub fn new(
        processor: Processor,
        analyst: Analyst,
        filter: Filter,
        geometry: GeometryConfig,
        screen_config: ScreenConfig,
    ) -> Self {
        let chunk_map = Self::init_chunk_map(geometry, screen_config);
        let output_buffer = vec![RGBPixel::black(); chunk_map.len()];

        // TODO:
        // 1) добавить расчёт размеров фрагмента
        return Self {
            processor,
            analyst,
            filter,
            chunk_map,
            output_buffer,
        };
    }

    fn init_chunk_map(geometry: GeometryConfig, screen_config: ScreenConfig) -> Vec<ChunkTask> {
        let mut chunk_map = Vec::<ChunkTask>::new();

        // Координаты заданы в виде:
        // ------> x
        // |
        // |
        // y

        //======================================//
        // ====== К О Э Ф Ф И Ц И Е Н Т Ы ===== //
        //======================================//
        // Коэффициент по вертикали
        let k_y =
            calculate_mm_to_px_k(screen_config.frame_height_mm, screen_config.frame_height_px);
        // Коэффициент по горизонтали (на случай нестандартных экранов)
        let k_x = calculate_mm_to_px_k(screen_config.frame_width_mm, screen_config.frame_width_px);

        //======================================//
        // === О Б Щ И Е  К О Н С Т А Н Т Ы === //
        //======================================//

        // Длина блока светодиодов по вертикали в пикселях
        let led_length_y_px = geometry.led_pos.led_length.as_pixels(k_y);
        // Длина блока светодиодов по горизонтали в пикселях
        let led_length_x_px = geometry.led_pos.led_length.as_pixels(k_x);

        // Количество пикселей в строке
        let px_in_row = screen_config.frame_width_px;

        //======================================//
        // ============= В Е Р Х ============== //
        //======================================//
        // Расчёт положения верхней ленты, начиная с верхнего левого угла

        // x = просто `отступ от ленты до края экрана слева`
        let start_up_x_px = geometry.led_pos.horizontal_offset.as_pixels(k_x);
        // y = `отступ от грани экрана` - `глубина чтения к краям`
        let start_up_y_px = Pixels::new(
            ((geometry.led_pos.gap.0 - geometry.reading.deep_out.0) as f64 * k_y).round() as usize,
        );

        for idx in 0..geometry.led_pos.horizontal_led_amount {
            // Изменяем x, добавляя к `стартовому значению` произведение `длины
            // блока светодиодов по x` и `порядкового номера блока`
            let current_up_x_px = Pixels::new(start_up_x_px.0 + (led_length_x_px.0 * idx));

            // `y` в данном случае не изменяется
            chunk_map.push(ChunkTask {
                // TODO: считывать кол-во полей в пикселе из конфига
                start_index: calculate_px_x_y_to_bytes(
                    current_up_x_px,
                    start_up_y_px,
                    px_in_row,
                    4,
                ),
                orientation: Orientation::Horizontal, // горизонтальные, т.к. они на горизонтальной ленте
            });
        }

        //======================================//
        // ============ П Р А В О ============= //
        //======================================//
        // Расчёт положения правой ленты (если смотреть на экран монитора),
        // начиная с левого верхнего угла

        // x = `длина экрана` - `отступ от грани экрана` - `глубина чтения к центру`
        let start_right_x_px = Pixels::new(
            ((screen_config.frame_width_mm.0 - geometry.led_pos.gap.0 - geometry.reading.deep_in.0)
                as f64
                * k_x)
                .round() as usize,
        );

        // y = просто `отступ от края экрана`
        let start_right_y_px = geometry.led_pos.vertical_offset.as_pixels(k_y);

        for idx in 0..geometry.led_pos.vertical_led_amount {
            // Изменяем `y`, добавляя к `стартовому значению` произведение
            // `длины блока светодиодов по y` и `порядкового номера блока`
            let current_right_y_px = Pixels::new(start_right_y_px.0 + (led_length_y_px.0 * idx));

            // `x` в данном случае не изменяется
            chunk_map.push(ChunkTask {
                // TODO: считывать кол-во полей в пикселе из конфига
                start_index: calculate_px_x_y_to_bytes(
                    start_right_x_px,
                    current_right_y_px,
                    px_in_row,
                    4,
                ),
                orientation: Orientation::Vertical, // вертикальные, т.к. они на вертикальной ленте
            });
        }

        //======================================//
        // =============== Н И З ============== //
        //======================================//
        // Расчёт положения нижней ленты, начиная с левого верхнего угла
        // **самого правого блока**
        // Здесь инвертируем порядок чтения. Читаем справа налево

        // x = `ширина экрана` - `отступ от края экрана` - `ширина блока
        // светодиодов`
        let start_down_x_px = Pixels::new(
            ((screen_config.frame_width_mm.0
                - geometry.led_pos.horizontal_offset.0
                - geometry.led_pos.led_length.0) as f64
                * k_x)
                .round() as usize,
        );

        // y = `длина экрана` - `отступ от грани экрана` - `глубина чтения к
        // центру`
        let start_down_y_px = Pixels::new(
            ((screen_config.frame_height_mm.0 - geometry.led_pos.gap.0 - geometry.reading.deep_in.0)
                as f64
                * k_y)
                .round() as usize,
        );

        for idx in 0..geometry.led_pos.horizontal_led_amount {
            // Изменяем x, вычитая из `стартового значения` произведение `длины
            // блока светодиодов по x` и `порядкового номера блока`
            let current_down_x_px = Pixels::new(start_down_x_px.0 - (led_length_x_px.0 * idx));

            // `y` в данном случае не изменяется
            chunk_map.push(ChunkTask {
                // TODO: считывать кол-во полей в пикселе из конфига
                start_index: calculate_px_x_y_to_bytes(
                    current_down_x_px,
                    start_down_y_px,
                    px_in_row,
                    4,
                ),
                orientation: Orientation::Horizontal, // горизонтальные, т.к. они на горизонтальной ленте
            });
        }

        //======================================//
        // ============= Л Е В О ============== //
        //======================================//
        // Расчёт положения левой ленты (если смотреть на экран монитора),
        // начиная с левого верхнего угла **самого нижнего блока**
        // Здесь инвертируем порядок чтения. Читаем снизу вверх

        // x = `отступ от грани экрана` - `глубина чтения к краям`
        let start_left_x_px = Pixels::new(
            ((geometry.led_pos.gap.0 - geometry.reading.deep_out.0) as f64 * k_x).round() as usize,
        );

        // y = `высота экрана` - `отступ от края экрана` - `ширина блока
        // светодиодов`
        let start_left_y_px = Pixels::new(
            ((screen_config.frame_height_mm.0
                - geometry.led_pos.vertical_offset.0
                - geometry.led_pos.led_length.0) as f64
                * k_y)
                .round() as usize,
        );

        for idx in 0..geometry.led_pos.vertical_led_amount {
            // Изменяем `y`, вычитая из `стартового значения` произведение
            // `длины блока светодиодов по y` и `порядкового номера блока`
            let current_left_y_px = Pixels::new(start_left_y_px.0 - (led_length_y_px.0 * idx));

            // `x` в данном случае не изменяется
            chunk_map.push(ChunkTask {
                // TODO: считывать кол-во полей в пикселе из конфига
                start_index: calculate_px_x_y_to_bytes(
                    start_left_x_px,
                    current_left_y_px,
                    px_in_row,
                    4,
                ),
                orientation: Orientation::Vertical, // вертикальные, т.к. они на вертикальной ленте
            });
        }

        // Возвращаем рассчитанную карту
        return chunk_map;
    }

    /// Обработка кадра
    ///
    /// Данный метод вызывается для обработки кадра и использует предоставленные
    /// методы обработки фрагмента и анализа цвета
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]] - указатель на кадр (массив пикселей)
    ///
    /// **Выходные поля:**
    /// - &[[RGBPixel]] - указатель на вычисленный массив цветов
    pub fn process_frame(&mut self, byte_frame: &[u8]) -> &[RGBPixel] {
        // Обрабатываем каждый фрагмент, используя предоставленный метод и
        // сохранённое в карте значение
        for (chunk_task, led_color) in self.chunk_map.iter().zip(self.output_buffer.iter_mut()) {
            // Сбрасываем анализ
            self.analyst.clear();

            // Вызываем обработчик кадра
            self.processor
                .process_chunk(byte_frame, *chunk_task, &mut self.analyst);

            // Сохраняем результат анализа
            *led_color = convert_hsv_to_rgb(self.analyst.get_winner());
        }

        return self.filter.apply(&self.output_buffer);
    }
}