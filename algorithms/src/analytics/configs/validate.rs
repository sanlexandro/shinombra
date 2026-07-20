//! Проверка конфигов

use super::HistogramConfig;
use common::configs::*;

impl ConfigValidate for HistogramConfig {
    /// Проверка [HistogramConfig]
    ///
    /// **Проверки:**
    /// - `0 < precision_level <= 20` - уровень точности не может быть =0 или
    ///   >20, т.к. он отвечает за количество разбиений окружности на 6. Т.е.
    ///   при разбиении с `precision_level = 1` получим ровно 6 гистограмм.
    ///   Алгоритм не может работать при 0 гистограмм, а 120 гистограмм - это
    ///   разумный предел, ведь на 3 градусах человек уже почти не различает разницы
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.precision_level == 0 || self.precision_level > 20 {
            return Err(ValidationError::InvalidValue {
                section: "color_histogram_config",
                field: "precision_level",
                message: format!(
                    "precision_level should be >0 and <20! precision_level in config: {}",
                    self.precision_level
                ),
            });
        }
        Ok(Vec::new())
    }
}
