//! Проверка конфигурации

use std::{format, net::ToSocketAddrs};

use common::configs::{ConfigValidate, ValidationError, ValidationWarning};

use crate::{ddp::config::config::DdpConfig, wled_drgb::types::DEFAULT_PORT};

impl ConfigValidate for DdpConfig {
    /// Проверка [DdpConfig]
    ///
    /// **Проверки:**
    /// - ip правильно десериализуется
    /// - `timeout == 0` - предупреждение о возможной некорректной работе
    /// - если установили mtu, просто предупреждение о возможной некорректной работе
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if let Err(err) =
            format!("{}:{}", self.ip, self.port.unwrap_or(DEFAULT_PORT)).to_socket_addrs()
        {
            return Err(ValidationError::InvalidValue {
                section: "ddp_config",
                field: "ip",
                message: format!("Ip can not be reached: {}.", err.to_string()),
            });
        }

        let mut result = Vec::new();

        if self.timeout.unwrap_or(100) == 0 {
            result.push(ValidationWarning::InvalidValue {
                section: "ddp_config",
                field: "timeout",
                message: "With zero timeout led can work with errors.".to_string(),
            });
        }

        if self.mtu.is_some() {
            result.push(ValidationWarning::InvalidValue {
                section: "ddp_config",
                field: "mtu",
                message: "Setting another MTU can break protocol work. Be careful with it.".to_string(),
            });
        }

        Ok(result)
    }
}
