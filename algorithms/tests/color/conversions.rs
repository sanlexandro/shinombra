//! Тест преобразования цветов

use algorithms::color::{
    conversion::{convert_hsv_to_rgb, convert_rgb_to_hsv},
    types::{HSVPixel, RGBPixel},
};

use crate::EPSILON;

/// Тест преобразования формата RGB в HSVs
///
/// Чёрный  (0;   0;   0)   -> (0.0;   0.0;   0.0)
/// Красный (255; 0;   0)   -> (0.0;   1.0; 255.0)
/// Зелёный (0;   255; 0)   -> (120.0; 1.0; 255.0)
/// Синий   (0;   0;   255) -> (240.0; 1.0; 255.0)
/// Белый   (255; 255; 255) -> (0.0;   0.0; 255.0)
#[test]
fn test_rgb_to_hsv() {
    let rgb_values = [
        (0, 0, 0),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 255, 255),
    ];

    let hsv_values = [
        (0.0, 0.0, 0.0),
        (0.0, 1.0, 255.0),
        (120.0, 1.0, 255.0),
        (240.0, 1.0, 255.0),
        (0.0, 0.0, 255.0),
    ];

    let tests_names = ["Black", "Red", "Green", "Blue", "White"];

    for ((rgb_value, hsv_value), test_name) in rgb_values
        .iter()
        .zip(hsv_values.iter())
        .zip(tests_names.iter())
    {
        let rgb = RGBPixel {
            red: rgb_value.0,
            green: rgb_value.1,
            blue: rgb_value.2,
        };
        let hsv = convert_rgb_to_hsv(rgb);

        assert!(
            (hsv.hue - hsv_value.0).abs() < EPSILON,
            "In conversion {} RGB -> HSV: hue is {}, waiting {}",
            test_name,
            hsv.hue,
            hsv_value.0
        );
        assert!(
            (hsv.saturation - hsv_value.1).abs() < EPSILON,
            "In conversion {} RGB -> HSV: saturation is {}, waiting {}",
            test_name,
            hsv.saturation,
            hsv_value.1
        );
        assert!(
            (hsv.value - hsv_value.2).abs() < EPSILON,
            "In conversion {} RGB -> HSV: value is {}, waiting {}",
            test_name,
            hsv.value,
            hsv_value.2
        );
    }
}

/// Тест преобразования формата HSV в RGB
///
/// Чёрный  (0.0;   0.0; 0.0)   -> (0;   0;   0)
/// Красный (0.0;   1.0; 255.0) -> (255; 0;   0)
/// Зелёный (120.0; 1.0; 255.0) -> (0;   255; 0)
/// Синий   (240.0; 1.0; 255.0) -> (0;   0;   255)
/// Белый   (0.0;   0.0; 1.0)   -> (255; 255; 255)
#[test]
fn test_hsv_to_rgb() {
    let hsv_values = [
        (0.0, 0.0, 0.0),
        (0.0, 1.0, 255.0),
        (120.0, 1.0, 255.0),
        (240.0, 1.0, 255.0),
        (0.0, 0.0, 255.0),
    ];

    let rgb_values = [
        (0, 0, 0),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 255, 255),
    ];

    let tests_names = ["Black", "Red", "Green", "Blue", "White"];

    for ((hsv_value, rgb_value), test_name) in hsv_values
        .iter()
        .zip(rgb_values.iter())
        .zip(tests_names.iter())
    {
        let hsv = HSVPixel {
            hue: hsv_value.0,
            saturation: hsv_value.1,
            value: hsv_value.2,
        };

        let rgb = convert_hsv_to_rgb(hsv);

        assert!(
            rgb.red == rgb_value.0,
            "In conversion {} HSV -> RGB: red is {}, waiting {}",
            test_name,
            rgb.red,
            rgb_value.0
        );
        assert!(
            rgb.green == rgb_value.1,
            "In conversion {} HSV -> RGB: green is {}, waiting {}",
            test_name,
            rgb.green,
            rgb_value.1
        );
        assert!(
            rgb.blue == rgb_value.2,
            "In conversion {} HSV -> RGB: blue is {}, waiting {}",
            test_name,
            rgb.blue,
            rgb_value.2
        );
    }
}

/// Тест двойного преобразования
#[test]
fn test_double_conversion() {
    for red in 0..255 {
        for green in 0..255 {
            for blue in 0..255 {
                let rgb = RGBPixel { red, green, blue };

                let new_rgb = convert_hsv_to_rgb(convert_rgb_to_hsv(rgb));

                assert!(rgb == new_rgb, "In double conversion RGB -> HSV -> RGB: rgb != new_rgb!\nrgb_new is ({};{};{}) waiting ({};{};{})", new_rgb.red, new_rgb.green, new_rgb.blue, rgb.red, rgb.green, rgb.blue);
            }
        }
    }
}
