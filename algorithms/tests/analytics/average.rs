//! Тестирование среднего арифметического

use algorithms::{
    analytics::{types::Average, ColorAnalyst},
    color::types::RGBPixel,
};

/// Тест среднего арифметического с одним заполнением цветом
///
/// В результате должны получить тот же цвет, что и передали в анализаторы
///
/// Чёрный  -> Чёрный
/// Красный -> Красный
/// Зелёный -> Зелёный
/// Синий   -> Синий
/// Белый   -> Белый
#[test]
fn test_average_with_one_color() {
    let colors = [
        RGBPixel {
            red: 0,
            green: 0,
            blue: 0,
        },
        RGBPixel {
            red: 255,
            green: 0,
            blue: 0,
        },
        RGBPixel {
            red: 0,
            green: 255,
            blue: 0,
        },
        RGBPixel {
            red: 0,
            green: 0,
            blue: 255,
        },
        RGBPixel {
            red: 255,
            green: 255,
            blue: 255,
        },
    ];

    for color in colors {
        let mut analytics = Average::new();

        analytics.add_data(color);

        let result = analytics.get_winner();

        assert!(
            color.red == result.red,
            "In Average after pushing one color ({}, {}, {}): red is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.red,
            color.red
        );
        assert!(
            color.green == result.green,
            "In Average after pushing one color ({}, {}, {}): green is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.green,
            color.green
        );
        assert!(
            color.blue == result.blue,
            "In Average after pushing one color ({}, {}, {}): blue is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.blue,
            color.blue
        );
    }
}

/// Тест среднего арифметического с одним заполнением цветом после очистки
///
/// Для теста аналитик заполняется белым цветом, затем очищается, а после
/// заполняется тестовым цветом. В результате должны получить как раз тестовый
/// цвет.
///
/// Чёрный  -> Чёрный
/// Красный -> Красный
/// Зелёный -> Зелёный
/// Синий   -> Синий
#[test]
fn test_average_with_one_color_after_clearing() {
    let colors = [
        RGBPixel {
            red: 0,
            green: 0,
            blue: 0,
        },
        RGBPixel {
            red: 255,
            green: 0,
            blue: 0,
        },
        RGBPixel {
            red: 0,
            green: 255,
            blue: 0,
        },
        RGBPixel {
            red: 0,
            green: 0,
            blue: 255,
        },
    ];

    for color in colors {
        let mut analytics = Average::new();

        analytics.add_data(RGBPixel::black());

        analytics.clear();

        analytics.add_data(color);

        let result = analytics.get_winner();

        assert!(
            color.red == result.red,
            "In Average after clearing and pushing one color ({}, {}, {}): red is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.red,
            color.red
        );
        assert!(
            color.green == result.green,
            "In Average after clearing and pushing one color ({}, {}, {}): green is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.green,
            color.green
        );
        assert!(
            color.blue == result.blue,
            "In Average after clearing and pushing one color ({}, {}, {}): blue is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.blue,
            color.blue
        );
    }
}

/// Тест переполнения среднего арифметического
///
/// Заполним анализатор 1000 белых пикселей
/// В результате должны получить белый
#[test]
fn test_average_overflow() {
    let wight = RGBPixel {
        red: 255,
        green: 255,
        blue: 255,
    };

    let mut analytic = Average::new();

    for _ in 0..1000 {
        analytic.add_data(wight);
    }

    let result = analytic.get_winner();

    assert!(
        wight == result,
        "In Average overflow test: result is ({}, {}, {}), waiting ({}, {}, {})",
        result.red,
        result.green,
        result.blue,
        wight.red,
        wight.green,
        wight.blue
    );
}

/// Тест вычисления среднего арифметического из одного цвета и белого
#[test]
fn test_average_with_mixing_color_and_wight() {
    let colors = [
        RGBPixel {
            red: 0,
            green: 0,
            blue: 0,
        },
        RGBPixel {
            red: 255,
            green: 0,
            blue: 0,
        },
        RGBPixel {
            red: 0,
            green: 255,
            blue: 0,
        },
        RGBPixel {
            red: 0,
            green: 0,
            blue: 255,
        },
        RGBPixel {
            red: 255,
            green: 255,
            blue: 255,
        },
    ];

    for color in colors {
        let mut analytics = Average::new();

        analytics.add_data(color);
        analytics.add_data(RGBPixel {
            red: 255,
            green: 255,
            blue: 255,
        });

        let result = analytics.get_winner();

        let avr_color = RGBPixel {
            red: ((color.red as u16 + 255) / 2) as u8,
            green: ((color.green as u16 + 255) / 2) as u8,
            blue: ((color.blue as u16 + 255) / 2) as u8,
        };

        assert!(
            avr_color.red == result.red,
            "In Average after pushing color ({}, {}, {}) with wight: red is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.red,
            avr_color.red
        );
        assert!(
            avr_color.green == result.green,
            "In Average after pushing color ({}, {}, {}) with wight: green is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.green,
            avr_color.green
        );
        assert!(
            avr_color.blue == result.blue,
            "In Average after pushing color ({}, {}, {}) with wight: blue is {}, waiting {}",
            color.red,
            color.green,
            color.blue,
            result.blue,
            avr_color.blue
        );
    }
}
