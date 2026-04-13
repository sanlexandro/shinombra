//! Тестирование движка обработки кадров (ColorEngine)
//!
//! Данные тесты проверяют встроенную логику основного алгоритма обхода кадра, а именно:
//!  * правильность расчёта размеров буферов
//!  * точность и корректность математического расчёта карты фрагментов (смещения в байтах и ориентация для всех четырёх граней экрана)
//!  * корректность цикла обработки всего кадра (правильное взаимодействие с `ChunkProcessor` и `ColorAnalyst`)
//!  * корректность вызова постобработки для полученных результатов (взаимодействие с `ColorFilter`)

use algorithms::{
    analytics::ColorAnalyst,
    color::types::{HSVPixel, RGBPixel},
    filters::ColorFilter,
    processing::{
        configs::{
            ChunkTask, GeometryConfig, LedPositionConfig, ScreenConfig, ScreenReadingConfig,
        },
        types::ColorEngine,
        ChunkProcessor,
    },
    units::{Millimeters, Orientation, Pixels},
};

//==============================================//
// ================ М О К И =================== //
//==============================================//

/// Фейковый обработчик фрагмента
#[derive(Clone, Default)]
struct MockChunkProcessor {
    pub recorded_tasks: std::rc::Rc<std::cell::RefCell<Vec<ChunkTask>>>,
}

impl ChunkProcessor for MockChunkProcessor {
    fn process_chunk<Analyst: ColorAnalyst>(
        &self,
        _byte_frame: &[u8],
        chunk_task: ChunkTask,
        _analyst: &mut Analyst,
    ) {
        self.recorded_tasks.borrow_mut().push(chunk_task);
    }
}

/// Фейковый анализатор цвета
#[derive(Clone)]
struct MockColorAnalyst {
    pub returned_color: HSVPixel,
}

impl Default for MockColorAnalyst {
    fn default() -> Self {
        Self {
            returned_color: HSVPixel {
                hue: 0.0,
                saturation: 0.0,
                value: 0.0,
            },
        }
    }
}

impl MockColorAnalyst {
    pub fn new(returned_color: HSVPixel) -> Self {
        Self { returned_color }
    }
}

impl ColorAnalyst for MockColorAnalyst {
    fn clear(&mut self) {}

    fn add_data(&mut self, _hsv: HSVPixel) {}

    fn get_winner(&mut self) -> HSVPixel {
        self.returned_color.clone()
    }
}

/// Фейковый фильтр
#[derive(Clone, Default)]
struct MockColorFilter {
    pub dummy_buffer: Vec<RGBPixel>,
    pub last_supplied_colors: std::rc::Rc<std::cell::RefCell<Vec<RGBPixel>>>,
}

impl ColorFilter for MockColorFilter {
    fn apply(&mut self, raw_colors: &[RGBPixel]) -> &[RGBPixel] {
        *self.last_supplied_colors.borrow_mut() = raw_colors.to_vec();

        if self.dummy_buffer.is_empty() {
            self.dummy_buffer = raw_colors.to_vec();
        }

        &self.dummy_buffer
    }
}

//==============================================//
// ================ Т Е С Т Ы ================= //
//==============================================//

/// Проверка инициализации базового класса `ColorEngine`
///
/// Данный тест проверяет, что размер получившегося массива `chunk_map` и `output_buffer`
/// равен суммарному количеству светодиодов на всех четырёх гранях экрана,
/// рассчитываемых по формуле: `2 * horizontal_led_amount + 2 * vertical_led_amount`.
#[test]
fn test_color_engine_new_sizes() {
    let screen = ScreenConfig {
        frame_width_px: Pixels::new(100),
        frame_height_px: Pixels::new(100),
        frame_width_mm: Millimeters::new(100),
        frame_height_mm: Millimeters::new(100),
    };

    let geometry = GeometryConfig {
        led_pos: LedPositionConfig {
            gap: Millimeters::new(0),
            vertical_offset: Millimeters::new(0),
            horizontal_offset: Millimeters::new(0),
            led_length: Millimeters::new(1),
            vertical_led_amount: 5,
            horizontal_led_amount: 10,
        },
        reading: ScreenReadingConfig {
            deep_in: Millimeters::new(0),
            deep_out: Millimeters::new(0),
        },
    };

    let processor = MockChunkProcessor::default();
    let analyst = MockColorAnalyst::default();
    let filter = MockColorFilter::default();

    let mut engine = ColorEngine::new(processor.clone(), analyst, filter, geometry, screen);

    let dummy_frame = [0];
    engine.process_frame(&dummy_frame);

    let expected_led_amount = 2 * 10 + 2 * 5; // 30
    assert_eq!(processor.recorded_tasks.borrow().len(), expected_led_amount);

    // Но также можно проверить, что фильтр получит буфер правильного размера
    let filtered_buffer = engine.apply_filters();
    assert_eq!(filtered_buffer.len(), expected_led_amount);
}

/// Проверка корректности расчёта карты фрагментов
///
/// Этот тест проверяет, что метод `init_chunk_map` правильно вычисляет
/// координаты (start_index) и ориентацию каждого фрагмента на четырёх гранях.
///
/// **Поля таблицы:**
/// - `idx` - порядковый номер блока в массиве `chunk_map`
/// - `expected_start_index` - ожидаемое значение байтового смещения
/// - `expected_orientation` - ожидаемая ориентация (горизонтальная или вертикальная)
#[test]
fn test_init_chunk_map_calculations() {
    // Удобные значения: 1 мм = 10 пикселей (по X и Y)
    let screen = ScreenConfig {
        frame_width_px: Pixels::new(1000),      // ширина в px (1000)
        frame_height_px: Pixels::new(1000),     // высота в px (1000)
        frame_width_mm: Millimeters::new(100),  // ширина в mm  (100) -> 10 px/mm
        frame_height_mm: Millimeters::new(100), // высота в mm  (100) -> 10 px/mm
    };

    let geometry = GeometryConfig {
        led_pos: LedPositionConfig {
            gap: Millimeters::new(5),                // отступ от края: 5мм * 10 = 50px
            vertical_offset: Millimeters::new(10),   // отступ сверху/снизу: 10мм * 10 = 100px
            horizontal_offset: Millimeters::new(20), // отступ слева/справа: 20мм * 10 = 200px
            led_length: Millimeters::new(30),        // длина блока: 30мм * 10 = 300px
            horizontal_led_amount: 2,                // 2 блока по горизонтали
            vertical_led_amount: 2,                  // 2 блока по вертикали
        },
        reading: ScreenReadingConfig {
            deep_in: Millimeters::new(2),  // отступ внутрь экрана: 2мм * 10 = 20px
            deep_out: Millimeters::new(1), // отступ к краям: 1мм * 10 = 10px
        },
    };

    // Коэффициенты px_in_row = 1000, bytes_in_px = 4
    // px_in_row * 4 = 4000 байт на строку

    // ВЕРХ:
    // start_y = (gap - deep_out) * 10 = (5 - 1) * 10 = 40
    // start_x (idx 0) = horizontal_offset * 10 = 20 * 10 = 200
    // start_x (idx 1) = 200 + led_length * 10 = 200 + 300 = 500
    //   -> ID 0: start_index = x*4 + y*4000 = 200*4 + 40*4000 = 800 + 160000 = 160_800
    //   -> ID 1: start_index = x*4 + y*4000 = 500*4 + 40*4000 = 2000 + 160000 = 162_000

    // ПРАВО:
    // start_x = width - gap - deep_in = (100 - 5 - 2) * 10 = 930
    // start_y (idx 0) = vertical_offset * 10 = 10 * 10 = 100
    // start_y (idx 1) = 100 + led_length * 10 = 100 + 300 = 400
    //   -> ID 2: start_index = x*4 + y*4000 = 930*4 + 100*4000 = 3720 + 400000 = 403_720
    //   -> ID 3: start_index = x*4 + y*4000 = 930*4 + 400*4000 = 3720 + 1600000 = 1_603_720

    // НИЗ: (читаем справа налево)
    // start_y = height - gap - deep_in = (100 - 5 - 2) * 10 = 930
    // start_x (idx 0) = width - horizontal_offset - led_length = 100 - 20 - 30 = 50 -> 500 px.
    // start_x (idx 1) = 500 - led_length = 500 - 300 = 200
    //   -> ID 4: start_index = x*4 + y*4000 = 500*4 + 930*4000 = 2000 + 3720000 = 3_722_000
    //   -> ID 5: start_index = x*4 + y*4000 = 200*4 + 930*4000 = 800 + 3720000 = 3_720_800

    // ЛЕВО: (читаем снизу вверх)
    // start_x = gap - deep_out = 5 - 1 = 4 -> 40 px.
    // start_y (idx 0) = height - vertical_offset - led_length = 100 - 10 - 30 = 60 -> 600 px.
    // start_y (idx 1) = 600 - led_length = 600 - 300 = 300
    //   -> ID 6: start_index = x*4 + y*4000 = 40*4 + 600*4000 = 160 + 2400000 = 2_400_160
    //   -> ID 7: start_index = x*4 + y*4000 = 40*4 + 300*4000 = 160 + 1200000 = 1_200_160

    let processor = MockChunkProcessor::default();
    let analyst = MockColorAnalyst::default();
    let filter = MockColorFilter::default();

    let mut engine = ColorEngine::new(processor.clone(), analyst, filter, geometry, screen);

    let dummy_frame = [0];
    engine.process_frame(&dummy_frame);
    let tasks = processor.recorded_tasks.borrow();

    let cases = vec![
        // (idx, expected_start_index, expected_orientation)
        (0, 160800, Orientation::Horizontal),
        (1, 162000, Orientation::Horizontal),
        (2, 403720, Orientation::Vertical),
        (3, 1603720, Orientation::Vertical),
        (4, 3722000, Orientation::Horizontal),
        (5, 3720800, Orientation::Horizontal),
        (6, 2400160, Orientation::Vertical),
        (7, 1200160, Orientation::Vertical),
    ];

    for (idx, expected_start_index, expected_orientation) in cases {
        let chunk = tasks[idx];
        assert_eq!(
            chunk.start_index, expected_start_index,
            "Failed on idx {} mapped to start index",
            idx
        );
        assert_eq!(
            chunk.orientation, expected_orientation,
            "Failed on idx {} mapped to orientation",
            idx
        );
    }
}

/// Проверка метода обработки кадра (`process_frame`)
///
/// Этот тест проверяет, что `process_frame` вызывает обработчик для каждого
/// фрагмента, очищает аналитику и записывает вычисленный цвет в буфер.
#[test]
fn test_process_frame() {
    let screen = ScreenConfig::default();
    let geometry = GeometryConfig {
        led_pos: LedPositionConfig {
            horizontal_led_amount: 1,
            vertical_led_amount: 0,
            ..Default::default()
        },
        reading: ScreenReadingConfig::default(),
    };

    let processor = MockChunkProcessor::default();
    // Настроим анализатор таким образом, чтобы он всегда возвращал чистый красный
    let analyst = MockColorAnalyst::new(HSVPixel {
        hue: 0.0,
        saturation: 1.0,
        value: 255.0,
    });

    // Мы хотим проверить именно внутренний буфер, который уходит фильтру, поэтому:
    let filter = MockColorFilter::default();

    let mut engine = ColorEngine::new(processor.clone(), analyst, filter.clone(), geometry, screen);

    // Создаем фиктивный кадр
    let dummy_frame: [u8; 1] = [0];

    engine.process_frame(&dummy_frame);

    // Должно быть вызвано 2 раза (так как 1+1 по горизонтали и 0 по вертикали => 2 элемента в chunk_map)
    let tasks = processor.recorded_tasks.borrow();
    assert_eq!(tasks.len(), 2);

    let _filtered = engine.apply_filters();
    let applied = filter.last_supplied_colors.borrow();

    // Анализатор должен был вернуть красный для всех фрагментов,
    // что конвертируется в RGB(255, 0, 0)
    for color in applied.iter() {
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
    }
}

/// Проверка применения фильтров (`apply_filters`)
///
/// Этот тест проверяет, что `ColorEngine::apply_filters` возвращает то
/// же самое значение, что генерируется его внутренним фильтром на основе
/// буфера из памяти класса.
#[test]
fn test_apply_filters() {
    let dummy_out = vec![RGBPixel {
        red: 10,
        green: 20,
        blue: 30,
    }];

    let filter = MockColorFilter {
        dummy_buffer: dummy_out.clone(),
        last_supplied_colors: Default::default(),
    };

    let screen = ScreenConfig::default();
    let geometry = GeometryConfig {
        led_pos: LedPositionConfig {
            horizontal_led_amount: 0,
            vertical_led_amount: 0,
            ..Default::default()
        },
        reading: ScreenReadingConfig::default(),
    };

    let mut engine = ColorEngine::new(
        MockChunkProcessor::default(),
        MockColorAnalyst::default(),
        filter,
        geometry,
        screen,
    );

    let result = engine.apply_filters();

    assert_eq!(result.len(), dummy_out.len());
    assert_eq!(result[0].red, dummy_out[0].red);
    assert_eq!(result[0].green, dummy_out[0].green);
    assert_eq!(result[0].blue, dummy_out[0].blue);
}
