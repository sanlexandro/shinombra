//! Тестирование проверки конфигурации для анализаторов

use algorithms::analytics::configs::ColorHistogramConfig;
use common::configs::ConfigValidate;

/// Тест валидации конфига для гистограмм
///
/// precision_level: 0  -> false
/// precision_level: 2  -> true
/// precision_level: 20 -> true
/// precision_level: 25 -> false
#[test]
fn test_color_histogram_validation_config() {
    let tests = [
        (ColorHistogramConfig { precision_level: 0 }, false),
        (ColorHistogramConfig { precision_level: 2 }, true),
        (
            ColorHistogramConfig {
                precision_level: 20,
            },
            true,
        ),
        (
            ColorHistogramConfig {
                precision_level: 25,
            },
            false,
        ),
    ];

    for test in tests.iter() {
        let val = test.0.validate();

        assert!(
            val.is_ok() == test.1,
            "In analytics configs ColorHistogramConfig for precision_level: {} validation is {}, waiting {}", test.0.precision_level, val.is_ok(), test.1
        );
    }
}
