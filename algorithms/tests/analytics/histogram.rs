//! Тестирование построения гистограмм и обработки голосований (histogram)
//!
//! Данные тесты проверяют алгоритмы накопления статистики по цветам:
//!  * инициализацию и очистку корзин (отдельных сегментов гистограммы)
//!  * правильное распределение голосующих пикселей по секторам (корзинам) в зависимости от цвета
//!  * корректную работу ахроматического сектора (36), который теперь отвечает за белый и серый цвета
//!  * определение сектора-победителя и вычисление усредненного цвета

use algorithms::{
    analytics::types::{ColorBin, ColorHistogram},
    color::types::HSVPixel,
};
use crate::test_fns::*;

/// Проверка базовых методов структуры `ColorBin`
#[test]
fn test_color_bin_lifecycle() {
    let mut bin = ColorBin::new();

    // Проверка состояния после new()
    assert_eq!(bin.weight, 0);
    assert_eq!(bin.votes, 0);
    assert_eq!(bin.sum_pixel.hue, 0.0);
    assert_eq!(bin.sum_pixel.saturation, 0.0);
    assert_eq!(bin.sum_pixel.value, 0.0);

    // Вносим мусорные данные
    bin.weight = 100;
    bin.votes = 5;
    bin.sum_pixel = HSVPixel {
        hue: 10.0,
        saturation: 1.0,
        value: 255.0,
    };

    // Проверяем сброс
    bin.clear();

    assert_eq!(bin.weight, 0);
    assert_eq!(bin.votes, 0);
    assert_eq!(bin.sum_pixel.hue, 0.0);
}

/// Проверка распределения голосов в зависимости от свойств пикселя
///
/// Этот тест проверяет, что метод `process_vote` правильно классифицирует цвета.
/// Теперь белый цвет не превращается в черный, а возвращается как ахроматический (sat = 0).
#[test]
fn test_histogram_process_vote() {
    let cases = vec![
        // (pixel, expected_winner)
        // 1. Чёрный цвет (value < 0.05) -> летит в корзину 36. winner -> black
        (HSVPixel::black(), HSVPixel::black()),
        
        // 2. Белый цвет (saturation < 0.15) -> летит в корзину 36. 
        // ТАК КАК яркость высокая, алгоритм должен вернуть белый (sat: 0, val: avg)
        (
            HSVPixel {
                hue: 200.0,
                saturation: 0.0,
                value: 255.0,
            },
            HSVPixel {
                hue: 0.0,
                saturation: 0.0,
                value: 255.0,
            },
        ),
        
        // 3. Тусклый серый цвет (saturation < 0.15) -> летит в 36.
        // Должен вернуться серый (sat: 0) с сохраненной яркостью.
        (
            HSVPixel {
                hue: 50.0,
                saturation: 0.1,
                value: 80.0,
            },
            HSVPixel {
                hue: 0.0,
                saturation: 0.0,
                value: 80.0,
            },
        ),
        
        // 4. Обычный яркий цвет -> попадает в свой сектор (45.0 / 10 = сектор 4)
        (
            HSVPixel {
                hue: 45.0,
                saturation: 1.0,
                value: 255.0,
            },
            HSVPixel {
                hue: 45.0,
                saturation: 1.0,
                value: 255.0,
            },
        ),
    ];

    for (pixel, expected_winner) in cases {
        let mut histogram = ColorHistogram::new();
        histogram.process_vote(pixel);

        let winner = histogram.determining_winner();
        assert_approximately_equal(winner.hue, expected_winner.hue, 0.001);
        assert_approximately_equal(winner.saturation, expected_winner.saturation, 0.001);
        assert_approximately_equal(winner.value, expected_winner.value, 0.001);
    }
}

/// Проверка метода очистки всей гистограммы
#[test]
fn test_histogram_clear() {
    let mut histogram = ColorHistogram::new();

    histogram.process_vote(HSVPixel {
        hue: 45.0,
        saturation: 1.0,
        value: 255.0,
    });

    let winner_before = histogram.determining_winner();
    assert!(winner_before.value > 0.0);

    histogram.clear();

    let winner_after = histogram.determining_winner();
    assert_approximately_equal(winner_after.value, 0.0, 0.001);
}

/// Проверка определения победителя (самого популярного цвета)
#[test]
fn test_determining_winner() {
    // Сценарий 1: Гистограмма пустая (должен вернуться чёрный цвет)
    let mut hist_empty = ColorHistogram::new();
    let winner = hist_empty.determining_winner();
    assert_approximately_equal(winner.value, 0.0, 0.001);

    // Сценарий 2: Соревнование между белым шумом и слабым цветом
    let mut hist_comp = ColorHistogram::new();
    // 5 черных пикселей (яркость 0.01). Вес в 36-м секторе: 5 * (0.01 * 100) = 5
    for _ in 0..5 {
        hist_comp.process_vote(HSVPixel { hue: 0.0, saturation: 0.0, value: 0.01 });
    }
    // 1 цветной пиксель. Вес: (11.0 * 1.0 * 100.0) = 1100
    hist_comp.process_vote(HSVPixel {
        hue: 0.0,
        saturation: 1.0,
        value: 11.0,
    });

    let winner = hist_comp.determining_winner();
    // Цвет должен победить черный шум
    assert_approximately_equal(winner.saturation, 1.0, 0.001);
    assert_approximately_equal(winner.value, 11.0, 0.001);

    // Сценарий 3: Явный победитель с расчетом среднего значения
    let mut hist_avg = ColorHistogram::new();
    // Сектор 12
    hist_avg.process_vote(HSVPixel { hue: 120.0, saturation: 1.0, value: 100.0 });
    hist_avg.process_vote(HSVPixel { hue: 126.0, saturation: 0.5, value: 200.0 });

    let winner = hist_avg.determining_winner();
    assert_approximately_equal(winner.hue, 123.0, 0.001);
    assert_approximately_equal(winner.saturation, 0.75, 0.001);
    assert_approximately_equal(winner.value, 150.0, 0.001);

    // Сценарий 4: Победа ахроматического сектора (Белый цвет)
    let mut hist_white = ColorHistogram::new();
    // Цветной пиксель (вес 50 * 1.0 * 100 = 5000)
    hist_white.process_vote(HSVPixel { hue: 200.0, saturation: 1.0, value: 50.0 });
    
    // Забиваем белым (100 пикселей, вес каждого 255 * 100 = 25500)
    for _ in 0..100 {
        hist_white.process_vote(HSVPixel { hue: 0.0, saturation: 0.0, value: 255.0 });
    }

    let winner = hist_white.determining_winner();
    // Теперь побеждает 36-й сектор, и он должен вернуть белый цвет
    assert_approximately_equal(winner.hue, 0.0, 0.001);
    assert_approximately_equal(winner.saturation, 0.0, 0.001);
    assert_approximately_equal(winner.value, 255.0, 0.001);
}