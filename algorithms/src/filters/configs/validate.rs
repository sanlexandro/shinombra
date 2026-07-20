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

impl ConfigValidate for SaturationBoostConfig {
    /// Проверка [SaturationBoostConfig]
    /// 
    /// **Проверки:**
    /// - `0 <= floor < 100` - порог должен быть строго в диапазоне (при =100% не
    ///   имеет смысла)
    /// - `boost_exponent >= 1` - обязательно для математики, иначе значения
    ///   пробьют диапазон
    /// - `boost_exponent > 3` - могут быть странные значение
    /// - `floor > 15%` - очень высокий порог
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.floor < 0.0 || self.floor >= 100.0 {
            return Err(ValidationError::InvalidValue {
                section: "saturation_boost_config",
                field: "floor",
                message: format!(
                    "Floor must be in [0%; 100%]! But it is {}%",
                    self.floor
                ),
            });
        }

        if self.boost_exponent < 1.0 {
            return Err(ValidationError::InvalidValue {
                section: "saturation_boost_config",
                field: "boost_exponent",
                message: format!(
                    "Boost exponent must be >= 1! But it is {}",
                    self.boost_exponent
                ),
            });
        }

        let mut vec: Vec<ValidationWarning> = Vec::new();

        if self.boost_exponent > 3.0 {
            vec.push(ValidationWarning::InvalidValue { 
                section: "saturation_boost_config", 
                field: "boost_exponent", 
                message: format!(
                    "Boost exponent should be in [1; 3], but you set {}. Colors can be kind of strange", 
                    self.boost_exponent
                ) 
            });
        } 

        if self.floor > 15.0 {
            vec.push(ValidationWarning::InvalidValue { 
                section: "saturation_boost_config", 
                field: "floor", 
                message: format!(
                    "Floor should be in [0%; 7%], but you set {}%. Colors can be kind of strange", 
                    self.floor
                ) 
            });
        }

        Ok(vec)
    }
}

impl ConfigValidate for WhiteBalanceConfig {
    /// Проверка [WhiteBalanceConfig]
    /// 
    /// **Проверки:**
    /// - `kelvins > 1000` - это ограничение алгоритма Таннера Хеллэнда
    /// - `kelvins > 20000` - могут быть очень странные цвета
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.kelvins < 1000 {
            return Err(ValidationError::InvalidValue {
                section: "white_balance_config",
                field: "kelvins",
                message: format!(
                    "Kelvins must be >= 1000! This is a limitation of the Tanner Helland algorithm. But it is {}K",
                    self.kelvins
                ),
            });
        }

        if self.kelvins > 20000 {
            return Ok(vec![ValidationWarning::InvalidValue { 
                section: "white_balance_config", 
                field: "kelvins", 
                message: format!(
                    "Kelvins should be in [1000; 20000], but you set {}K. Colors can be kind of strange", 
                    self.kelvins
                ) 
            }]);
        }

        Ok(vec![])
    }
}

impl ConfigValidate for ChannelGainConfig {
    /// Проверка [ChannelGainConfig]
    /// 
    /// **Проверки:**
    /// - все коэффициенты должны быть [0; 1]
    /// - если коэффициент меньше 0.7 - предупреждение
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.k_red < 0.0 || self.k_red > 1.0 {
            return Err(ValidationError::InvalidValue {
                section: "channel_gain_config",
                field: "k_red",
                message: format!(
                    "Coefficient must be in [0; 1]!. But it is {}",
                    self.k_red
                ),
            });
        }

        if self.k_green < 0.0 || self.k_green > 1.0 {
            return Err(ValidationError::InvalidValue {
                section: "channel_gain_config",
                field: "k_green",
                message: format!(
                    "Coefficient must be in [0; 1]!. But it is {}",
                    self.k_green
                ),
            });
        }

        if self.k_blue < 0.0 || self.k_blue > 1.0 {
            return Err(ValidationError::InvalidValue {
                section: "channel_gain_config",
                field: "k_blue",
                message: format!(
                    "Coefficient must be in [0; 1]!. But it is {}",
                    self.k_blue
                ),
            });
        }

        if self.k_red < 0.7 || self.k_green < 0.7 || self.k_blue < 0.7 {
            return Ok(vec![
                ValidationWarning::StructValue { 
                    section: "channel_gain_config", 
                    message: format!(
                        "Coefficients should be greater then 0.7, but you set r:{}, g:{}, b{}. Colors can be kind of strange", 
                        self.k_red, 
                        self.k_green, 
                        self.k_blue
                    ) 
            }]);
        }


       Ok(vec![]) 
    }
}