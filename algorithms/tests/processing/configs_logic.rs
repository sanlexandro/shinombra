//! Тестирование логики работы с конфигурациями (configs_logic)
//!
//! Данные тесты проверяют корректность вычисления вспомогательных параметров
//! на основе заданных пользователем или аппаратурой конфигураций:
//!  * расчёт количества светодиодов на экране
//!  * расчёт конфигурации единичного фрагмента для последующего захвата (ширина, высота)
//!  * обновление данных разрешения экрана `load_px`

use algorithms::{
    processing::configs::{GeometryConfig, LedPositionConfig, ScreenConfig, ScreenReadingConfig},
    units::{Millimeters, Pixels},
};

/// Проверка расчёта количества светодиодов в ленте
///
/// Метод должен возвращать сумму горизонтальных и вертикальных светодиодов,
/// умноженную на 2 (так как лента идет по всему периметру: 2 горизонтали и 2 вертикали).
///
/// **Поля таблицы:**
/// - `horizontal_led_amount` - количество диодов на верхней/нижней грани
/// - `vertical_led_amount` - количество диодов на левой/правой грани
/// - `expected_total` - ожидаемое суммарное количество диодов
#[test]
fn test_calculate_leds_amount() {
    let cases = vec![
        // (horizontal, vertical, expected_total)
        (10, 5, 30),  // (10 + 5) * 2 = 30
        (0, 0, 0),    // Нулевой экран
        (25, 12, 74), // (25 + 12) * 2 = 74
    ];

    for (horizontal, vertical, expected_total) in cases {
        let config = GeometryConfig {
            led_pos: LedPositionConfig {
                horizontal_led_amount: horizontal,
                vertical_led_amount: vertical,
                ..Default::default()
            },
            reading: ScreenReadingConfig::default(),
        };

        assert_eq!(
            config.calculate_leds_amount(),
            expected_total,
            "Failed on horizontal={} vertical={}",
            horizontal,
            vertical
        );
    }
}

/// Проверка расчёта конфигурации единичного фрагмента
///
/// Метод `calculate_chunk_config` вычисляет размеры одного фрагмента в пикселях:
/// - `width` вычисляется на основе длины диода (`led_length`) и коэффициента по ширине `k_x`.
/// - `height` вычисляется на основе суммы глубин захвата (`deep_in` + `deep_out`) и коэффициента по высоте `k_y`.
///
/// **Поля таблицы:**
/// - Конфиг экрана: `(w_mm, h_mm, w_px, h_px)` -> определяет $k_x$, $k_y$.
/// - Конфиг геометрии: `(led_len_mm, deep_in_mm, deep_out_mm)`.
/// - Ожидаемый результат: `(expected_w_px, expected_h_px)`.
#[test]
fn test_calculate_chunk_config() {
    let cases = vec![
        // k_x = 10 px/mm, k_y = 10 px/mm
        // (w_mm, h_mm, w_px, h_px), (led_len, deep_in, deep_out), (exp_w, exp_h)
        ((100, 100, 1000, 1000), (30, 2, 1), (300, 30)),
        // k_x = 2 px/mm, k_y = 5 px/mm (нестандартный пиксель)
        ((100, 100, 200, 500), (10, 5, 5), (20, 50)),
        // Нулевые глубины (например, только край экрана)
        ((100, 100, 1000, 1000), (45, 0, 0), (450, 0)),
    ];

    for ((w_mm, h_mm, w_px, h_px), (led_len, deep_in, deep_out), (exp_w, exp_h)) in cases {
        let geometry = GeometryConfig {
            led_pos: LedPositionConfig {
                led_length: Millimeters::new(led_len),
                ..Default::default()
            },
            reading: ScreenReadingConfig {
                deep_in: Millimeters::new(deep_in),
                deep_out: Millimeters::new(deep_out),
            },
        };

        let screen = ScreenConfig {
            frame_width_mm: Millimeters::new(w_mm),
            frame_height_mm: Millimeters::new(h_mm),
            frame_width_px: Pixels::new(w_px),
            frame_height_px: Pixels::new(h_px),
        };

        let chunk_config = geometry.calculate_chunk_config(screen);

        assert_eq!(
            chunk_config.width,
            Pixels::new(exp_w),
            "Failed width calculation for k_x"
        );
        assert_eq!(
            chunk_config.height,
            Pixels::new(exp_h),
            "Failed height calculation for k_y"
        );
    }
}

/// Проверка обновления базового разрешения экрана (пикселей) через `load_px`
///
/// Метод `load_px` принимает значения `u32` и безопасно конвертирует их
/// в `Pixels(usize)`.
///
/// **Поля таблицы:**
/// - `w_px` - задаваемая ширина в пикселях
/// - `h_px` - задаваемая высота в пикселях
#[test]
fn test_screen_config_load_px() {
    let cases = vec![
        (1920, 1080),
        (2560, 1440),
        (800, 600),
        (0, 0), // Проверка нулевых значений
    ];

    for (w_px, h_px) in cases {
        let mut screen = ScreenConfig::default();
        screen.load_px(w_px, h_px);

        assert_eq!(
            screen.frame_width_px,
            Pixels::new(w_px as usize),
            "Failed w_px update"
        );
        assert_eq!(
            screen.frame_height_px,
            Pixels::new(h_px as usize),
            "Failed h_px update"
        );
    }
}