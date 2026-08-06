//! Валидация конфигурации

use std::vec;

use common::configs::{ConfigValidate, ValidationError, ValidationWarning};

use crate::config::Settings;

impl ConfigValidate for Settings {
    /// Проверка [Settings]
    ///
    /// **Проверки:**
    /// - `max_fps != 0` - защита от деления на ноль!
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.max_fps.unwrap_or(60) == 0 {
            return Err(ValidationError::InvalidValue {
                section: "settings",
                field: "max_fps",
                message: "Max FPS must be greater than 0! Zero division protection.".to_string(),
            });
        }

        Ok(vec![])
    }
}
