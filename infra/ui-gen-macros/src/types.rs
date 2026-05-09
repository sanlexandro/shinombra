//! Вспомогательные AST-структуры
//! 
//! Эти структуры описывают синтаксис макроса ui-генерации. 
//! Они используются для удобного парсинга входных данных.

use syn::{Expr, Ident, LitStr, Token, punctuated::Punctuated};

/// Настройки конфигурации для поля-реестра (выбор из enum).
pub(crate) struct RegistrySettings {
    pub(crate) file_path: String,
    pub(crate) registry_enum: Ident,
}

/// Настройки для обычного текстового поля ввода.
pub(crate) struct TextFieldSettings {
    pub(crate) placeholder: String,
}

/// Настройки для числового поля ввода, включая возможные ограничения.
pub(crate) struct NumericFieldSettings {
    pub(crate) min: Option<f32>,
    pub(crate) max: Option<f32>,
}

/// Настройки для булевого поля ввода (checkbox).
pub(crate) struct BoolFieldSettings;

/// Настройки для виджета ползунка (slider).
pub(crate) struct SliderFieldSettings {
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) step: f32,
}

/// Поддерживаемые типы виджетов веб-интерфейса.
pub(crate) enum WidgetType {
    Registry(RegistrySettings),
    TextField(TextFieldSettings),
    NumericField(NumericFieldSettings),
    BoolField(BoolFieldSettings),
    SliderField(SliderFieldSettings),
    Wrapper(Ident, Box<WidgetType>),
    WrapperVec(Box<WidgetType>),
}

/// Описание одного поля: к какой переменной оно привязано и какой виджет использует.
pub(crate) struct FieldSetting {
    pub(crate) field_name: Ident,
    pub(crate) widget_type: WidgetType,
}

/// Конфигурация отображения конкретной структуры целиком.
pub(crate) struct StructConfig {
    pub(crate) struct_name: Ident,
    pub(crate) fields: Punctuated<FieldSetting, Token![,]>,
    pub(crate) condition: Option<Expr>,
}

/// Общие входные данные макроса для генерации UI на основе файла.
pub(crate) struct UiGenInput {
    pub(crate) file_path: LitStr,
    pub(crate) configs: Punctuated<StructConfig, Token![,]>,
}

// Вспомогательная структура для парсинга нескольких путей
pub(crate) struct PathList {
    pub(crate) inputs: Vec<UiGenInput>,
}
