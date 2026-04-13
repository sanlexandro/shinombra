//! Тестирование алгоритма фильтрации значений с помощью экспоненциального
//! скользящего среднего (EMA)
//!
//! Данные тесты проверяют поведение фильтра в различных ситуациях. А именно:
//!  * при отсутствии изменения в цвете (чёрный и белый)
//!  * при резком изменении цвета (от чёрного к белому и наоборот)
//!  * при работе с массивом из нескольких независимых пикселей

use crate::test_fns::*;
use algorithms::{color::types::RGBPixel, filters::types::EmaFilter};

/// Проверка фильтра при отсутствии изменений цвета
///
/// Данный тест подаёт на вход фильтра один и тот же цвет 10 раз подряд.
/// Проверяется, что алгоритм не искажает цвет и не вызывает переполнения (overflow).
///
/// **Поля таблицы:**
/// - `fixed_color` - цвет для инициализации буфера и подачи на вход (RGBPixel)
#[test]
fn test_no_changes() {
    let cases = vec![
        // (fixed_color)
        (RGBPixel::black()), // Чёрный цвет
        (RGBPixel {
            red: 255,
            green: 255,
            blue: 255,
        }), // Белый цвет
    ];

    for fixed_color in cases {
        let color_vec = vec![fixed_color.clone(); 1];
        let mut filter = EmaFilter::new_with_state(color_vec.clone());

        for _ in 0..10 {
            let got_color = filter.process_ema(&color_vec);
            assert_eq!(fixed_color, got_color[0]);
        }
    }
}

/// Проверка фильтра при резком изменении цвета
///
/// Данный тест на каждом шаге подаёт целевой цвет до тех пор, пока буфер фильтра
/// полностью не станет целевого цвета. Проверяется, что сглаживание происходит
/// корректно без скачков и укладывается в заданное число кадров.
///
/// **Поля таблицы:**
/// - `initial_color` - стартовый цвет (каким инициализируется буфер)
/// - `target_color` - целевой цвет, который будет подаваться
/// - `max_steps` - за сколько максимально шагов фильтр должен дойти до цели
/// - `is_increasing` - флаг того, что яркость каналов должна возрастать на каждом шаге
#[test]
fn test_sudden_change() {
    let cases = vec![
        // (initial_color, target_color, max_steps, is_increasing)
        // От чёрного к белому
        (
            RGBPixel::black(),
            RGBPixel {
                red: 255,
                green: 255,
                blue: 255,
            },
            50,
            true,
        ),
        // От белого к чёрному
        (
            RGBPixel {
                red: 255,
                green: 255,
                blue: 255,
            },
            RGBPixel::black(),
            50,
            false,
        ),
    ];

    for (initial, target, max_steps, is_increasing) in cases {
        let initial_vec = vec![initial.clone(); 1];
        let target_vec = vec![target.clone(); 1];

        let mut filter = EmaFilter::new_with_state(initial_vec.clone());

        let mut counter: u32 = 0;
        let mut buffer = initial_vec.clone();

        while counter < 100 {
            // Условное большое число (лимит цикла)
            let got = filter.process_ema(&target_vec);

            if got[0] == target_vec[0] {
                break;
            }

            if is_increasing {
                assert_bigger(got[0].red, buffer[0].red);
                assert_bigger(got[0].green, buffer[0].green);
                assert_bigger(got[0].blue, buffer[0].blue);
            } else {
                assert_smaller(got[0].red, buffer[0].red);
                assert_smaller(got[0].green, buffer[0].green);
                assert_smaller(got[0].blue, buffer[0].blue);
            }

            counter += 1;
            buffer = got.to_vec();
        }

        assert_smaller(counter, max_steps);
    }
}

/// Проверка независимой обработки нескольких пикселей
///
/// Данный тест инициализирует буфер несколькими независимыми цветами (изображением).
/// Проверяется, что каналы разных пикселей не смешиваются между собой в процессе работы с EMA.
///
/// **Поля таблицы:**
/// - `colors` - вектор стартовых цветов
#[test]
fn test_multiple_pixels_independence() {
    let cases = vec![
        // (colors)
        vec![
            RGBPixel {
                red: 255,
                green: 0,
                blue: 0,
            }, // Красный
            RGBPixel {
                red: 0,
                green: 255,
                blue: 0,
            }, // Зелёный
            RGBPixel {
                red: 0,
                green: 0,
                blue: 255,
            }, // Синий
        ],
    ];

    for colors in cases {
        let mut filter = EmaFilter::new_with_state(colors.clone());

        for _ in 0..10 {
            let got_colors = filter.process_ema(&colors);

            for i in 0..colors.len() {
                assert_eq!(got_colors[i], colors[i]);
            }
        }
    }
}
