//! Проверка конфигов

use std::{format, vec};

use super::*;
use common::{configs::*, units::{logic::calculate_mm_to_px_k, *}};

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
    /// - проверка расчёта всех границ
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

        let geometry = &self.geometry_config;
        let screen = &self.screen_config;

        // Коэффициенты
        let k_x = calculate_mm_to_px_k(screen.frame_width_mm, screen.frame_width_px);
        let k_y = calculate_mm_to_px_k(screen.frame_height_mm, screen.frame_height_px);

        let frame_w = screen.frame_width_px.0;
        let frame_h = screen.frame_height_px.0;

        let led_len_x = geometry.led_pos.led_length.as_pixels(k_x).0;
        let led_len_y = geometry.led_pos.led_length.as_pixels(k_y).0;

        // Вспомогательная макро-/функция проверки одного прямоугольника
        let check_rect = |side: &str, x: usize, y: usize, w: usize, h: usize| -> Result<(), ValidationError> {
            if x >= frame_w || y >= frame_h {
                return Err(ValidationError::StructValue {
                    section: "[screen_config] + [led_position_config] + [screen_reading_config]",
                    message: format!(
                        "Side {}: start position ({}, {}) is outside the frame ({}x{}). Check your parameters.",
                        side, x, y, frame_w, frame_h
                    ),
                });
            }
            if x + w > frame_w || y + h > frame_h {
                return Err(ValidationError::StructValue {
                    section: "[screen_config] + [led_position_config] + [screen_reading_config]",
                    message: format!(
                        "Side {}: fragment rect (x={}, y={}, w={}, h={}) goes outside the frame ({}x{}). Check your parameters.",
                        side, x, y, w, h, frame_w, frame_h
                    ),
                });
            }
            Ok(())
        };

        // ============================================================
        // UP
        // ============================================================
        {
            let horizontal_length_mm =
                Millimeters(geometry.led_pos.horizontal_led_amount as u32) * geometry.led_pos.led_length.0;
            let horizontal_offset_mm = (screen.frame_width_mm - horizontal_length_mm) / 2;

            let start_x = horizontal_offset_mm.as_pixels(k_x).0;
            let start_y = ((geometry.led_pos.gap.0 - geometry.reading.deep_out.0) as f64 * k_y)
                .round() as usize;

            // Высота фрагмента ≈ deep_in + deep_out (самый частый вариант)
            // Если у тебя другая формула размера фрагмента — подставь её сюда
            let frag_h = ((geometry.reading.deep_in.0 + geometry.reading.deep_out.0) as f64 * k_y)
                .round() as usize;
            let frag_h = frag_h.max(1); // защита от нуля

            for idx in 0..geometry.led_pos.horizontal_led_amount {
                let x = start_x + led_len_x * idx;
                check_rect("Up", x, start_y, led_len_x, frag_h)?;
            }
        }

        // ============================================================
        // RIGHT
        // ============================================================
        {
            let vertical_length_mm =
                Millimeters(geometry.led_pos.vertical_led_amount as u32) * geometry.led_pos.led_length;
            let vertical_offset_mm = (screen.frame_height_mm - vertical_length_mm) / 2;

            let start_x = ((screen.frame_width_mm.0
                - geometry.led_pos.gap.0
                - geometry.reading.deep_in.0) as f64
                * k_x)
                .round() as usize;
            let start_y = vertical_offset_mm.as_pixels(k_y).0;

            let frag_w = ((geometry.reading.deep_in.0 + geometry.reading.deep_out.0) as f64 * k_x)
                .round() as usize;
            let frag_w = frag_w.max(1);

            for idx in 0..geometry.led_pos.vertical_led_amount {
                let y = start_y + led_len_y * idx;
                check_rect("Right", start_x, y, frag_w, led_len_y)?;
            }
        }

        // ============================================================
        // DOWN
        // ============================================================
        {
            let horizontal_length_mm =
                Millimeters(geometry.led_pos.horizontal_led_amount as u32) * geometry.led_pos.led_length.0;
            let horizontal_offset_mm = (screen.frame_width_mm - horizontal_length_mm) / 2;

            let start_x = ((screen.frame_width_mm.0
                - horizontal_offset_mm.0
                - geometry.led_pos.led_length.0) as f64
                * k_x)
                .round() as usize;
            let start_y = ((screen.frame_height_mm.0
                - geometry.led_pos.gap.0
                - geometry.reading.deep_in.0) as f64
                * k_y)
                .round() as usize;

            let frag_h = ((geometry.reading.deep_in.0 + geometry.reading.deep_out.0) as f64 * k_y)
                .round() as usize;
            let frag_h = frag_h.max(1);

            for idx in 0..geometry.led_pos.horizontal_led_amount {
                // В коде идёт вычитание, поэтому проверяем в обратном порядке
                let x = start_x.saturating_sub(led_len_x * idx);
                check_rect("Down", x, start_y, led_len_x, frag_h)?;
            }
        }

        // ============================================================
        // LEFT
        // ============================================================
        {
            let vertical_length_mm =
                Millimeters(geometry.led_pos.vertical_led_amount as u32) * geometry.led_pos.led_length;
            let vertical_offset_mm = (screen.frame_height_mm - vertical_length_mm) / 2;

            let start_x = ((geometry.led_pos.gap.0 - geometry.reading.deep_out.0) as f64 * k_x)
                .round() as usize;
            let start_y = ((screen.frame_height_mm.0
                - vertical_offset_mm.0
                - geometry.led_pos.led_length.0) as f64
                * k_y)
                .round() as usize;

            let frag_w = ((geometry.reading.deep_in.0 + geometry.reading.deep_out.0) as f64 * k_x)
                .round() as usize;
            let frag_w = frag_w.max(1);

            for idx in 0..geometry.led_pos.vertical_led_amount {
                let y = start_y.saturating_sub(led_len_y * idx);
                check_rect("Left", start_x, y, frag_w, led_len_y)?;
            }
        }

        Ok(vec![])
    }
}