//! Проверка конфигов

use super::*;
use common::{configs::*, units::Pixels};

impl ConfigValidate for ChunkConfig {
    /// Проверка [ChunkConfig]
    ///
    /// **Проверки:**
    /// - `width` и `height` > 0. Зона захвата (Chunk) не может иметь нулевой размер,
    ///   так как это сделает невозможным расчет среднего цвета и может привести
    ///   к ошибкам при итерации по буферу кадра.
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        if self.width == Pixels(0) {
            return Err(ValidationError::InvalidValue {
                section: "ALGORITHMS",
                field: "chunk.width",
                message: "Capture zone width must be greater than 0 pixels. Sorry, it is algorithms error".into(),
            });
        }

        if self.height == Pixels(0) {
            return Err(ValidationError::InvalidValue {
                section: "ALGORITHMS",
                field: "chunk.height",
                message: "Capture zone height must be greater than 0 pixels. Sorry, it is algorithms error".into(),
            });
        }

        Ok(Vec::new())
    }
}

impl ConfigValidate for CheckerboardConfig {
    /// Проверка [CheckerboardConfig]
    ///
    /// **Проверки:**
    /// - `pixel_step` и `row_stride` > 0. Нулевой шаг приведет к зависанию алгоритма на одном пикселе.
    /// - `pixel_step` или `row_stride` > 15 (предупреждение). Слишком большой шаг сканирования
    ///   делает выборку цветов разреженной, что приводит к потере деталей и "шумному" результату.
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        let section = "checkerboard_config";

        if self.pixel_step == 0 {
            return Err(ValidationError::InvalidValue {
                section,
                field: "pixel_step",
                message: "Step cannot be zero (infinite loop protection)".into(),
            });
        }
        if self.row_stride == 0 {
            return Err(ValidationError::InvalidValue {
                section,
                field: "row_stride",
                message: "Stride cannot be zero (infinite loop protection)".into(),
            });
        }

        let mut warnings = Vec::new();

        if self.pixel_step > 15 {
            warnings.push(ValidationWarning::InvalidValue {
                section,
                field: "pixel_step",
                message: format!(
                    "Step {} is quite large. Average color calculation might become unstable",
                    self.pixel_step
                )
                .into(),
            });
        }
        if self.row_stride > 15 {
            warnings.push(ValidationWarning::InvalidValue {
                section,
                field: "row_stride",
                message: format!(
                    "Stride {} is quite large. Average color calculation might become unstable",
                    self.row_stride
                )
                .into(),
            });
        }

        Ok(warnings)
    }
}

impl ConfigValidate for DynamicCheckerboardConfig {
    /// Проверка [DynamicCheckerboardConfig]
    ///
    /// **Проверки:**
    /// - `pixel_step` и `row_stride` > 0. Нулевой шаг приведет к зависанию алгоритма на одном пикселе.
    /// - `pixel_step` или `row_stride` > 15 (предупреждение). Слишком большой шаг сканирования
    ///   делает выборку цветов разреженной, что приводит к потере деталей и "шумному" результату.
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError> {
        let section = "dynamic_checkerboard_config";

        if self.pixel_step == 0 {
            return Err(ValidationError::InvalidValue {
                section,
                field: "pixel_step",
                message: "Step cannot be zero (infinite loop protection)".into(),
            });
        }
        if self.row_stride == 0 {
            return Err(ValidationError::InvalidValue {
                section,
                field: "row_stride",
                message: "Stride cannot be zero (infinite loop protection)".into(),
            });
        }

        let mut warnings = Vec::new();

        if self.pixel_step > 15 {
            warnings.push(ValidationWarning::InvalidValue {
                section,
                field: "pixel_step",
                message: format!(
                    "Step {} is quite large. Average color calculation might become unstable",
                    self.pixel_step
                )
                .into(),
            });
        }
        if self.row_stride > 15 {
            warnings.push(ValidationWarning::InvalidValue {
                section,
                field: "row_stride",
                message: format!(
                    "Stride {} is quite large. Average color calculation might become unstable",
                    self.row_stride
                )
                .into(),
            });
        }

        Ok(warnings)
    }
}
