//! Проверка конфигов

use super::*;
use common::configs::*;

impl ConfigValidate for EmaFilterConfig {
    /// Проверка [EmaFilterConfig]
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

impl ConfigValidate for GammaFilterConfig {
    /// Проверка [GammaFilterConfig]
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
