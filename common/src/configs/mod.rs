//! Общая информация о конфигах

use logger::{log, LogLevel};

/// Возможные ошибки
///
/// **Варианты:**
/// - `InvalidValue` - индивидуальная ошибка (имя поля + сообщение)
/// - `RequiredField` - отсутствие поля (имя поля)
/// - `StructValue` - индивидуальная ошибка в логике структуры (сообщение)
pub enum ValidationError {
    InvalidValue {
        section: &'static str,
        field: &'static str,
        message: String,
    },
    RequiredField {
        section: &'static str,
        field: &'static str,
    },
    StructValue {
        section: &'static str,
        message: String,
    },
}

#[derive(Clone)]
pub enum ValidationWarning {
    InvalidValue {
        section: &'static str,
        field: &'static str,
        message: String,
    },
    StructValue {
        section: &'static str,
        message: String,
    },
}

/// Отображение ошибок
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidValue {
                section,
                field,
                message,
            } => {
                write!(f, "Critical error in [{}] {}: {}", section, field, message)
            }
            ValidationError::RequiredField { section, field } => {
                write!(
                    f,
                    "Missing required field '{}' in section [{}]",
                    field, section
                )
            }
            ValidationError::StructValue { section, message } => {
                write!(f, "Logic error in section [{}]: {}", section, message)
            }
        }
    }
}

/// Отображение предупреждений
impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationWarning::InvalidValue {
                section,
                field,
                message,
            } => {
                write!(
                    f,
                    "In [{}] '{}' has invalid value: {}",
                    section, field, message
                )
            }
            ValidationWarning::StructValue { section, message } => {
                write!(f, "In [{}] {}", section, message)
            }
        }
    }
}

/// Отображение всех предупреждений
pub fn warn_validate(warnings: Vec<ValidationWarning>, module: &str) {
    for warn in warnings {
        log!(LogLevel::Warn, module, "{}", warn);
    }
}

/// Проверка одного конфига
pub fn validate_config(config: &dyn ConfigValidate, module: &str) {
    config
        .validate()
        .map(|warnings| warn_validate(warnings, module))
        .unwrap_or_else(|e| {
            log!(LogLevel::Error, module, "{}", e);
            std::process::exit(1);
        });
}

/// Проверка массива конфигов
pub fn validate_configs(configs: &[&dyn ConfigValidate], module: &str) {
    for cfg in configs {
        validate_config(*cfg, module);
    }
}

/// Трейт проверки конфигурации
///
/// Необходим реализации единого метода проверки конфигурации
///
/// **Реализуемые методы:**
/// - `validate` - проверка
pub trait ConfigValidate {
    /// Проверка конфигурации
    ///
    /// Проверяет данные согласно внутренней логике
    ///
    /// **Выходные поля:**
    /// - [Result]<[Vec]<[ValidationWarning]>, [ValidationError]> - вектор предупреждений или ошибка
    fn validate(&self) -> Result<Vec<ValidationWarning>, ValidationError>;
}
