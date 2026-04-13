//! Тестирование построения гистограмм и обработки голосований (histogram)
//!
//! Данные тесты проверяют алгоритмы накопления статистики по цветам:
//!  * инициализацию и очистку корзин (отдельных сегментов гистограммы)
//!  * правильное распределение голосующих пикселей по секторам (корзинам) в зависимости от цвета
//!  * корректную фильтрацию чёрно-белых/тёмных оттенков в отдельную 'мусорную' корзину
//!  * определение сектора-победителя (самого популярного цвета) и вычисление усредненного цвета в нём

use algorithms::{
    analytics::types::{ColorBin, ColorHistogram},
    color::types::HSVPixel,
};
use crate::test_fns::*;

/// Проверка базовых методов структуры `ColorBin`
///
/// Тестируется, что корзина инициализируется нулевыми значениями,
/// и метод `clear` успешно сбрасывает все накопленные данные обратно в нуль.
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
/// Этот тест проверяет, что метод `process_vote` правильно учитывает цвет.
/// Вместо доступа к приватным полям, мы проверяем поведение через публичное API
/// `determining_winner()`. Если цвет мусорный - возвращается черный. Если нормальный - сам цвет.
///
/// **Поля таблицы:**
/// - `pixel` - входной пиксель для голосования
/// - `expected_winner` - ожидаемый победитель-цвет (отражающий правильность попадания в мусор или в сектор)
#[test]
fn test_histogram_process_vote() {
    let cases = vec![
        // (pixel, expected_winner)
        // 1. Чёрный цвет (value = 0) -> летит в корзину 36 (мусорную). winner -> black
        (HSVPixel::black(), HSVPixel::black()),
        // 2. Белый/Серый цвет (saturation = 0) -> летит в корзину 36 -> winner -> black
        (
            HSVPixel {
                hue: 200.0,
                saturation: 0.0,
                value: 255.0,
            },
            HSVPixel::black(),
        ),
        // 3. Очень тусклый цвет (sat*val < 10) -> вес 8. Летит в 36 -> winner -> black
        (
            HSVPixel {
                hue: 50.0,
                saturation: 0.1,
                value: 80.0,
            },
            HSVPixel::black(),
        ),
        // 4. Обычный цвет -> вес = 255. Попадает в свой сектор и становится победителем
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
        // 5. Цвет в другом секторе
        (
            HSVPixel {
                hue: 359.0,
                saturation: 1.0,
                value: 200.0,
            },
            HSVPixel {
                hue: 359.0,
                saturation: 1.0,
                value: 200.0,
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

    // Заполняем несколько случайных корзин
    histogram.process_vote(HSVPixel {
        hue: 45.0,
        saturation: 1.0,
        value: 255.0,
    });
    histogram.process_vote(HSVPixel {
        hue: 15.0,
        saturation: 1.0,
        value: 100.0,
    });

    // Убеждаемся, что накопились данные
    let winner_before = histogram.determining_winner();
    assert!(winner_before.value > 0.0, "Histogram should have some data");

    // Вызываем очистку (ожидаем сброса в мусор)
    histogram.clear();

    // Гистограмма пуста -> 0-й вес -> returns black
    let winner_after = histogram.determining_winner();
    assert_approximately_equal(winner_after.hue, 0.0, 0.001);
    assert_approximately_equal(winner_after.saturation, 0.0, 0.001);
    assert_approximately_equal(winner_after.value, 0.0, 0.001);
}

/// Проверка определения победителя (самого популярного цвета)
///
/// Тестируется логика нахождения сектора с максимальным весом и
/// расчёта усреднённого цвета внутри этой корзины.
#[test]
fn test_determining_winner() {
    // Сценарий 1: Гистограмма пустая (должен вернуться чёрный цвет)
    let mut hist_empty = ColorHistogram::new();
    let winner = hist_empty.determining_winner();
    assert_approximately_equal(winner.hue, 0.0, 0.001);
    assert_approximately_equal(winner.value, 0.0, 0.001);

    // Сценарий 2: Много 'чёрного' шума (сектор 36) и один слабый, но нормальный цвет
    // Нормальный цвет должен перевесить при правильном балансе, либо 36 выигрывает,
    // но мы проверим, что выигрывает именно цветной сектор, если его вес выше.
    let mut hist_black = ColorHistogram::new();
    for _ in 0..5 {
        // 5 мусорных голосов, вес каждого = 1. Итого вес 36-го = 5
        hist_black.process_vote(HSVPixel::black());
    }
    // 1 нормальный голос, но с малым весом = 11 (sat=1.0, val=11). Итого вес сектора 0 = 11
    hist_black.process_vote(HSVPixel {
        hue: 0.0,
        saturation: 1.0,
        value: 11.0,
    });

    let winner = hist_black.determining_winner();
    // Выигрывает сектор 0, так как у него вес 11 > 5 (один голос против 5 мусорных)
    assert_approximately_equal(winner.hue, 0.0, 0.001);
    assert_approximately_equal(winner.saturation, 1.0, 0.001);
    assert_approximately_equal(winner.value, 11.0, 0.001);

    // Сценарий 3: Явный победитель из нескольких голосов с расчетом среднего значения
    let mut hist_avg = ColorHistogram::new();
    // Сектор 12 (Hue ~ 120-129)
    hist_avg.process_vote(HSVPixel {
        hue: 120.0,
        saturation: 1.0,
        value: 100.0,
    }); // weight=100
    hist_avg.process_vote(HSVPixel {
        hue: 126.0,
        saturation: 0.5,
        value: 200.0,
    }); // weight=100

    // Сектор 20 (неудачник)
    hist_avg.process_vote(HSVPixel {
        hue: 205.0,
        saturation: 1.0,
        value: 50.0,
    }); // weight=50

    let winner = hist_avg.determining_winner();
    // Сектор 12 выиграл (суммарный вес: 200). Голосов: 2.
    // Среднее Hue = (120+126)/2 = 123.0
    // Среднее Sat = (1.0+0.5)/2 = 0.75
    // Среднее Val = (100+200)/2 = 150.0
    assert_approximately_equal(winner.hue, 123.0, 0.001);
    assert_approximately_equal(winner.saturation, 0.75, 0.001);
    assert_approximately_equal(winner.value, 150.0, 0.001);

    // Сценарий 4: Гарантированно выигрывает 36 (мусорный) сектор
    let mut hist_trash = ColorHistogram::new();
    hist_trash.process_vote(HSVPixel {
        hue: 205.0,
        saturation: 1.0,
        value: 50.0,
    }); // weight=50, индекс=20

    // Забиваем всё шумом с весом=1, чтобы перевесить 50
    for _ in 0..60 {
        hist_trash.process_vote(HSVPixel::black());
    } // итог мусорного веса = 60

    let winner = hist_trash.determining_winner();
    // Если выиграл 36-й корзин (шум/чёрный), результат должен быть строго (0.0, 0.0, 0.0)
    assert_approximately_equal(winner.hue, 0.0, 0.001);
    assert_approximately_equal(winner.saturation, 0.0, 0.001);
    assert_approximately_equal(winner.value, 0.0, 0.001);
}
