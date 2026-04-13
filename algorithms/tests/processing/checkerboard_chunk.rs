//! Тестирование алгоритма обхода фрагмента в шахматном порядке (`checkerboard_chunk`)
//!
//! Данные тесты проверяют логику обхода массива байтов фрагмента в шахматном порядке:
//!  * правильный мапинг параметров в зависимости от ориентации (get_params_for)
//!  * корректность смещения при построчном чтении (сам шахматный алгоритм сдвига)
//!  * правильность извлечения BGR значений из байтового массива и перевод их в HSV

use algorithms::{
    analytics::ColorAnalyst,
    color::{conversion::convert_hsv_to_rgb, types::HSVPixel},
    processing::{
        configs::{CheckerboardConfig, ChunkConfig, ChunkTask, ScreenConfig},
        types::CheckerboardScanner,
    },
    units::{Orientation, Pixels},
};

//==============================================//
// ================ М О К И =================== //
//==============================================//

/// Фейковый анализатор цвета для сбора статистики о считанных пикселях
#[derive(Clone, Default)]
struct TrackingColorAnalyst {
    pub recorded_hsves: std::rc::Rc<std::cell::RefCell<Vec<HSVPixel>>>,
}

impl ColorAnalyst for TrackingColorAnalyst {
    fn clear(&mut self) {
        self.recorded_hsves.borrow_mut().clear();
    }

    fn add_data(&mut self, hsv: HSVPixel) {
        self.recorded_hsves.borrow_mut().push(hsv);
    }

    fn get_winner(&mut self) -> HSVPixel {
        HSVPixel::black()
    }
}

//==============================================//
// ================ Т Е С Т Ы ================= //
//==============================================//

/// Проверка адаптации параметров обхода в зависимости от ориентации
///
/// Алгоритм "переворачивает" ширину и высоту при вертикальной ориентации,
/// а также меняет местами шаг строк и шаг пикселей.
///
/// **Поля таблицы:**
/// - `orientation` - текущая ориентация фрагмента
/// - `expected_chunk_w` - ожидаемая ширина фрагмента для обхода
/// - `expected_chunk_h` - ожидаемая высота
/// - `expected_row_stride` - ожидаемый шаг строк
/// - `expected_pixel_step` - ожидаемый шаг пикселей в строке
#[test]
fn test_checkerboard_config_get_params() {
    let config = CheckerboardConfig {
        config: ChunkConfig {
            width: Pixels::new(10),
            height: Pixels::new(20),
        },
        pixel_step: 2,
        row_stride: 4,
    };

    let cases = vec![
        // (orientation, expected_chunk_w, expected_chunk_h, expected_row_stride, expected_pixel_step)
        // Для горизонтальной ленты:
        (Orientation::Horizontal, 10, 20, 4, 2),
        // Для вертикальной ленты:
        (Orientation::Vertical, 20, 10, 2, 4),
    ];

    for (orientation, exp_w, exp_h, exp_row_stride, exp_pixel_step) in cases {
        let (w, h, row_stride, pixel_step) = config.get_params_for(orientation);

        assert_eq!(w, exp_w, "Failed width on {:?}", orientation);
        assert_eq!(h, exp_h, "Failed height on {:?}", orientation);
        assert_eq!(row_stride, exp_row_stride, "Failed row_stride on {:?}", orientation);
        assert_eq!(pixel_step, exp_pixel_step, "Failed pixel_step on {:?}", orientation);
    }
}

/// Проверка логики самого шахматного обхода
///
/// Данный тест создает фейковый буфер кадра экрана, заполняя его цветами в строго определенных
/// пикселях. Проверяется, что сканер при соответствующих параметрах извлечет именно "наши"
/// пиксели и преобразует BGR в RGB -> HSV корректно.
#[test]
fn test_process_checkerboard_chunk_logic() {
    // Экран: 4x4 пикселя (16 пикселей = 64 байта)
    let screen_config = ScreenConfig {
        frame_width_px: Pixels::new(4),
        frame_height_px: Pixels::new(4),
        ..Default::default()
    };

    // Настраиваем фрагмент с шахматным обходом:
    // Фрагмент размером 4x4, шаг по строкам 2, шаг по пикселям 2.
    // Шахматный порядок означает, что:
    // На 0-й строке мы читаем: пиксели 0, 2
    // На 2-й строке (смещение из-за нечетной итерации idx/stride = 2/2 = 1) мы читаем: пиксели 1, 3
    let alg_config = CheckerboardConfig {
        config: ChunkConfig {
            width: Pixels::new(4),
            height: Pixels::new(4),
        },
        pixel_step: 2,
        row_stride: 2,
    };

    let mut byte_frame = vec![0u8; 4 * 4 * 4];

    // Помощник для записи целевого пикселя. Формат массива - BGR + Alpha (4 байта)
    let mut set_pixel = |x: usize, y: usize, r: u8, g: u8, b: u8| {
        let index = (y * 4 + x) * 4;
        byte_frame[index] = b; // Blue  (алгоритм читает bgr[0])
        byte_frame[index + 1] = g; // Green (алгоритм читает bgr[1])
        byte_frame[index + 2] = r; // Red   (алгоритм читает bgr[2])
        byte_frame[index + 3] = 0; // Alpha (игнорируется)
    };

    // Строка 0 (Y=0): пиксели X=0 и X=2
    set_pixel(0, 0, 255, 0, 0); // Красный (попадёт в анализ)
    set_pixel(2, 0, 0, 255, 0); // Зеленый (попадёт в анализ)

    // Строка 2 (Y=2): пиксели X=1 и X=3 (т.к. сканер сделает смещение pixel_step/2 = 1)
    set_pixel(1, 2, 0, 0, 255); // Синий (попадёт в анализ)
    set_pixel(3, 2, 255, 255, 0); // Желтый (попадёт в анализ)

    let scanner = CheckerboardScanner::new(alg_config, screen_config);
    let mut analyst = TrackingColorAnalyst::default();

    let task = ChunkTask {
        start_index: 0,
        orientation: Orientation::Horizontal,
    };

    // Вызываем сканер
    scanner.process_checkerboard_chunk(&byte_frame, task, &mut analyst);

    // Извлекаем результаты. Для удобства проверки переведем обратно в RGB
    let recorded = analyst.recorded_hsves.borrow();

    // Ожидаем ровно 4 считанных пикселя
    assert_eq!(
        recorded.len(),
        4,
        "Should have read exactly 4 pixels due to checkerboard pattern"
    );

    let c0 = convert_hsv_to_rgb(recorded[0].clone());
    assert_eq!(
        (c0.red, c0.green, c0.blue),
        (255, 0, 0),
        "First pixel should be red"
    );

    let c1 = convert_hsv_to_rgb(recorded[1].clone());
    assert_eq!(
        (c1.red, c1.green, c1.blue),
        (0, 255, 0),
        "Second pixel should be green"
    );

    let c2 = convert_hsv_to_rgb(recorded[2].clone());
    assert_eq!(
        (c2.red, c2.green, c2.blue),
        (0, 0, 255),
        "Third pixel should be blue"
    );

    let c3 = convert_hsv_to_rgb(recorded[3].clone());
    assert_eq!(
        (c3.red, c3.green, c3.blue),
        (255, 255, 0),
        "Fourth pixel should be yellow"
    );
}
