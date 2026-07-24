//! Проверка конфигов

use std::{format, vec};

use super::*;
use common::{configs::*, units::*};

impl ConfigValidate for ScreenConfig {
    /// Проверка [ScreenConfig]
    ///
    /// **Проверки:**
    /// - `frame_width_px`, `frame_height_px` > 0 - иначе сообщение об ошибке в
    ///   логике работы программы
    /// - `frame_width_mm`, `frame_height_mm` > 0
    /// - Соотношение px/mm (коэффициент плотности) для X и Y должно быть ~одинаковым.
    ///   Разница более 1% вызывает предупреждение, так как это обычно указывает на
    ///   ошибку в физических размерах монитора.
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        let section = "screen_config";
        // Проверка физических и пиксельных размеров
        if self.frame_width_px == Pixels(0) {
            return Err(ValidationError::InvalidValue {
                section,
                field: "frame_width_px",
                message: "Fatal algorithmic error. Width in pixels must be greater than 0, \
                          but capture thread give zero value"
                    .into(),
            });
        }
        if self.frame_height_px == Pixels(0) {
            return Err(ValidationError::InvalidValue {
                section,
                field: "frame_height_px",
                message: "Fatal algorithmic error. Height in pixels must be greater than 0, \
                          but capture thread give zero value"
                    .into(),
            });
        }
        if self.frame_width_mm == Millimeters(0) {
            return Err(ValidationError::InvalidValue {
                section,
                field: "frame_width_mm",
                message: "Width in millimeters must be greater than 0".into(),
            });
        }
        if self.frame_height_mm == Millimeters(0) {
            return Err(ValidationError::InvalidValue {
                section,
                field: "frame_height_mm",
                message: "Height in millimeters must be greater than 0".into(),
            });
        }

        // Проверка плотности пикселей (Pixel Aspect Ratio)
        // Разница между коэффициентами по X и Y не должна превышать 0.01 (1%)
        let x_k = self.calculate_x_k();
        let y_k = self.calculate_y_k();

        if (x_k - y_k).abs() > 0.02 {
            return Ok(vec![ValidationWarning::StructValue{
                section,
                message: "Significant difference between horizontal and vertical pixel density detected. \
                          Please verify physical monitor dimensions in [screen_config]."
                    .into(),
            }
             ]);
        }

        Ok(vec![])
    }
}

impl ConfigValidate for LedPositionConfig {
    /// Проверка [LedPositionConfig]
    ///
    /// **Проверки:**
    /// - `vertical_led_amount` и `horizontal_led_amount` > 0
    /// - `led_length` > 0
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        let section = "led_position_config";

        if self.vertical_led_amount == 0 {
            return Err(ValidationError::InvalidValue {
                section,
                field: "vertical_led_amount",
                message: "Vertical led amount must be greater than 0".into(),
            });
        }
        if self.horizontal_led_amount == 0 {
            return Err(ValidationError::InvalidValue {
                section,
                field: "horizontal_led_amount",
                message: "Horizontal led amount must be greater than 0".into(),
            });
        }

        if self.led_length == Millimeters(0) {
            return Err(ValidationError::InvalidValue {
                section,
                field: "led_length",
                message: "Led length amount must be greater than 0 (infinite loop protection)"
                    .into(),
            });
        }

        Ok(vec![])
    }
}

impl ConfigValidate for ScreenReadingConfig {
    /// Проверка [ScreenReadingConfig]
    ///
    /// **Проверки:**
    /// - `deep_in` и `deep_out` не могут `= 0` одновременно - фрагменты будут пустыми
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.deep_in == Millimeters(0) && self.deep_out == Millimeters(0) {
            return Err(ValidationError::StructValue {
                section: "screen_reading_config",
                message: "Deep in and out must NOT be zero simultaneously \
                          fragments will be empty (empty loop protection)"
                    .into(),
            });
        }

        Ok(vec![])
    }
}

impl ConfigValidate for GeometryPlusScreenConfig {
    /// Проверка [GeometryPlusScreenConfig]
    ///
    /// **Проверки:**
    /// - лента не выходит за границы экрана!
    /// - зона считывания не выходит за границы экрана
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        let geometry = self.geometry_config;
        let screen = self.screen_config;

        let vertical_led_length =
            Millimeters(geometry.led_pos.vertical_led_amount as u32) * geometry.led_pos.led_length;
        let horizontal_led_length = Millimeters(geometry.led_pos.horizontal_led_amount as u32)
            * geometry.led_pos.led_length;

        // Горизонтальная часть ленты должна быть короче или равна ширине экрана
        if horizontal_led_length > screen.frame_width_mm {
            return Err(ValidationError::StructValue {
                section: "[screen_config] + [led_position_config]",
                message: format!(
                    "Horizontal led length must be <= than screen width \
                     horizontal_led_amount * led_length = {} * {} = {}; \
                     frame_width_mm = {}; {} > {}!",
                    geometry.led_pos.horizontal_led_amount,
                    geometry.led_pos.led_length,
                    horizontal_led_length,
                    screen.frame_width_mm,
                    horizontal_led_length,
                    screen.frame_width_mm
                ),
            });
        }

        // Вертикальная часть ленты должна быть короче или равна высоте экрана
        if vertical_led_length > screen.frame_height_mm {
            return Err(ValidationError::StructValue {
                section: "[screen_config] + [led_position_config]",
                message: format!(
                    "Vertical led length must be <= than screen height \
                     vertical_led_amount * led_length = {} * {} = {}; \
                     frame_height_mm = {}; {} > {}!",
                    geometry.led_pos.vertical_led_amount,
                    geometry.led_pos.led_length,
                    vertical_led_length,
                    screen.frame_height_mm,
                    vertical_led_length,
                    screen.frame_height_mm
                ),
            });
        }

        // Зона чтения из цента экрана не может быть больше отступа от края
        // экрана до ленты
        if geometry.reading.deep_out > geometry.led_pos.gap {
            return Err(ValidationError::StructValue {
                section: "[screen_config] + [screen_reading_config]",
                message: format!(
                    "The reading area from the center of the screen cannot be larger \
                     than the gap from the edge of the screen to the led. \
                     deep_out = {}; gap = {}; {} > {}!",
                    geometry.reading.deep_out,
                    geometry.led_pos.gap,
                    geometry.reading.deep_out,
                    geometry.led_pos.gap
                ),
            });
        }

        // Зона чтения вглубь экрана не может быть больше, чем самая короткая
        // часть экрана без одного отступа от края экрана до ленты
        let shorter_side = screen.frame_width_mm.min(screen.frame_height_mm);
        if geometry.reading.deep_in > (shorter_side - geometry.led_pos.gap) {
            return Err(ValidationError::StructValue {
                section: "[screen_config] + [screen_reading_config]",
                message: format!(
                    "The reading area deep into the screen cannot be larger \
                     than the shortest part of the screen without one gap \
                     from the edge of the screen to the led. \
                     shorter_side - gap = {} - {} = {}; deep_in = {}; {} > {}!",
                    shorter_side,
                    geometry.led_pos.gap,
                    shorter_side - geometry.led_pos.gap,
                    geometry.reading.deep_in,
                    geometry.reading.deep_in,
                    shorter_side - geometry.led_pos.gap
                ),
            });
        }

        Ok(vec![])
    }
}
