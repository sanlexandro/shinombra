//! Реализация трейта для состояний

use crate::processing::processors::{types::EmptyState, ChunkState};

/// Для пустого состояния
impl ChunkState for EmptyState {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self()
    }
}
