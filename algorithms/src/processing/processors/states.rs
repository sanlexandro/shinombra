//! Реализация трейта для состояний

use crate::processing::processors::{
    types::{CrawlCheckerboardState, EmptyState},
    ChunkState,
};

/// Для пустого состояния
impl ChunkState for EmptyState {
    /// Конструктор
    fn new() -> Self
    where
        Self: Sized,
    {
        Self()
    }
}

/// Для состояние в динамической шахматке
impl ChunkState for CrawlCheckerboardState {
    /// Конструктор
    ///
    /// Создаёт структуру с нулевыми полями
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            current_column: 0,
            current_row: 0,
        }
    }
}
