//! Проверка конфигов

use std::{format, vec};

use super::*;
use common::configs::*;

impl ConfigValidate for EmaConfig {
    /// Проверка [EmaConfig]
    ///
    /// **Проверки:**
    /// - `0.0 < alpha <= 1.0` - коэффициент сглаживания должен находиться в этом пределе.
    ///   При `alpha = 0` алгоритм перестает учитывать новые кадры (свет "замерзает"),
    ///   а при `alpha < 0` система становится нестабильной и значения улетают в бесконечность.
    ///   Значение `1.0` означает полное отсутствие сглаживания.
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.alpha <= 0.0 {
            return Err(ValidationError::InvalidValue {
                section: "ema_filter_config",
                field: "alpha",
                message:
                    "Alpha must be greater than 0. Current value leads to frozen or unstable light."
                        .to_string(),
            });
        }

        if self.alpha > 1.0 {
            return Err(ValidationError::InvalidValue {
                section: "ema_filter_config",
                field: "alpha",
                message: "Alpha cannot exceed 1.0 (1.0 means no smoothing).".to_string(),
            });
        }

        Ok(Vec::new())
    }
}

impl ConfigValidate for GammaConfig {
    /// Проверка [GammaConfig]
    ///
    /// **Проверки:**
    /// - `gamma > 0.0` - коэффициент гаммы не может быть нулевым или отрицательным.
    ///   Математически `x^0 = 1` (весь экран станет белым), а отрицательная гамма
    ///   приведет к делению на ноль при появлении черных пикселей.
    /// - `gamma > 3.0` (Warning) - значения выше 3 считаются экстремальными.
    ///   Цвета могут стать слишком темными и неестественными, так как кривая
    ///   становится слишком крутой.
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.gamma <= 0.0 {
            return Err(ValidationError::InvalidValue {
                section: "gamma_filter_config",
                field: "gamma",
                message: format!(
                    "Gamma must be greater than 0.0. Current value ({}) will break color calculations.",
                    self.gamma
                ),
            });
        }

        if self.gamma > 3.0 {
            return Ok(vec![ValidationWarning::InvalidValue {
                section: "gamma_filter_config",
                field: "gamma",
                message: format!(
                    "Usually gamma <= 3, but you set: {}. Colors could be kind a strange",
                    self.gamma
                ),
            }]);
        }

        Ok(vec![])
    }
}

impl ConfigValidate for BlackThresholdConfig {
    /// Проверка [BlackThresholdConfig]
    ///
    /// **Проверки:**
    /// - `0 <= threshold <= 100` - отсечка не может быть вне диапазона %
    /// - `0 <= fade_range <= 100` - ширина плавного диапазона не может быть
    ///   больше самого диапазона %
    /// - `threshold + fade_range <= 100` - диапазон работы фильтра не может
    ///   быть больше диапазона Value в HSV
    /// - `falloff_exponent < 0` - увеличенные значения выйдут за диапазон
    /// - `falloff_exponent < 1 || falloff_exponent > 3` - предупреждение о
    ///   странной работе
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.threshold < 0.0 || self.threshold > 100.0 {
            return Err(ValidationError::InvalidValue {
                section: "black_threshold_config",
                field: "threshold",
                message: format!(
                    "Threshold must be in [0%; 100%]! But it is {}%",
                    self.threshold
                ),
            });
        }

        if self.fade_range < 0.0 || self.fade_range > 100.0 {
            return Err(ValidationError::InvalidValue {
                section: "black_threshold_config",
                field: "fade_range",
                message: format!(
                    "Fade range must be in [0%; 100%]! But it is {}%",
                    self.fade_range
                ),
            });
        }

        if self.threshold + self.fade_range > 100.0 {
            return Err(ValidationError::StructValue {
                section: "black_threshold_config",
                message: format!(
                    "Range is too wide! `Threshold` + `fade range` is {}% + {} = {}%, what is greater then 100%. Lower threshold or fade range", 
                    self.threshold, 
                    self.fade_range, 
                    self.threshold + self.fade_range),
            });
        }

        if self.falloff_exponent < 0.0 {
             return Err(ValidationError::InvalidValue {
                section: "black_threshold_config",
                field: "falloff_exponent",
                message: format!(
                    "Falloff exponent must be in >0! But it is {}",
                    self.falloff_exponent
                ),
            });
        }

        if self.falloff_exponent < 1.0 || self.falloff_exponent > 3.0 {
            return Ok(vec![ValidationWarning::InvalidValue { 
                section: "black_threshold_config", 
                field: "falloff_exponent", 
                message: format!(
                    "Falloff exponent should be in [1; 3], but you set {}. Colors can be kind of strange", 
                    self.falloff_exponent
                ) 
            }]);
        }

        Ok(vec![])
    }
}
