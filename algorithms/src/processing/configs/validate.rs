//! Проверка конфигов

use super::*;
use common::{configs::*, units::*};

impl ConfigValidate for ScreenConfig {
    /// Проверка [ScreenConfig]
    ///
    /// **Проверки:**
    /// - `frame_width_px`, `frame_height_px` > 0
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
                message: "Width in pixels must be greater than 0".into(),
            });
        }
        if self.frame_height_px == Pixels(0) {
            return Err(ValidationError::InvalidValue {
                section,
                field: "frame_height_px",
                message: "Height in pixels must be greater than 0".into(),
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

        Ok(Vec::new())
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

        Ok(Vec::new())
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

        Ok(Vec::new())
    }
}
