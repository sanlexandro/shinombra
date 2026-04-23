//! Объявление различных вариантов фильтрации цвета

#[derive(Debug, Clone)]
pub enum ColorFilterType {
    NoFilter,
    EmaFilter,
}