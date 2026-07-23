//! Проверка конфигурации

use std::{format, net::ToSocketAddrs};

use common::configs::{ConfigValidate, ValidationError, ValidationWarning};

use crate::wled_drgb::{config::config::WledDrgbConfig, types::DEFAULT_PORT};

impl ConfigValidate for WledDrgbConfig {
    /// Проверка [WledDrgbConfig]
    ///
    /// **Проверки:**
    /// - ip правильно десериализуется
    /// - `timeout == 0` - предупреждение о возможной некорректной работе
    /// - `max_fps <= 10` - предупреждение о некорректной работе ленты
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if let Err(err) =
            format!("{}:{}", self.ip, self.port.unwrap_or(DEFAULT_PORT)).to_socket_addrs()
        {
            return Err(ValidationError::InvalidValue {
                section: "d_r_g_b_config",
                field: "ip",
                message: format!("Ip can not be reached: {}.", err.to_string()),
            });
        }

        let mut result = Vec::new();

        if self.timeout.unwrap_or(100) == 0 {
            result.push(ValidationWarning::InvalidValue {
                section: "d_r_g_b_config",
                field: "timeout",
                message: "With zero timeout led can work with errors.".to_string(),
            });
        }

        if self.max_fps.unwrap_or(60) <= 10 {
            result.push(ValidationWarning::InvalidValue {
                section: "d_r_g_b_config",
                field: "max_fps",
                message: "With small FPS led can work kind of strange.".to_string(),
            });
        }

        Ok(result)
    }
}
