//! Модуль обработки кадра
//! 
//! Данный модуль необходим для обработки кадра, посредством разделения его на
//! фрагменты и обработки их в дальнейшем
//! 
//! Методы обработки фрагмента:
//! - В шахматном порядке [types::CheckerboardScanner] - данный метод проходит
//!   по фрагменту в жёстко фиксированном шахматном порядке. Дальнейшая работа с
//!   анализом цвета ведётся с помощью переданного метода обработки из
//!   [crate::analytics]

pub mod checkerboard_chunk;
pub mod frame;
pub mod types;

use super::processing::types::{CheckerboardScanner, Orientation};
use crate::analytics::ColorAccumulator;

/// Трейт обработки фрагмента
///
/// **Методы:**
/// - `process_chunk` - метод обработки фрагмента
pub trait ChunkProcessor {
    /// Обработка фрагмента
    ///
    /// Этот метод вызывается для обрабоки кажодго фрагмента
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]]                  - указатель на кадр (массив
    ///   пикселей) 
    /// - `chunk_start_index`: [usize]           - индек пикселя в массиве, с
    ///   которого начинается фрагмент (*верхний правый угол*)
    /// - `chunk_orientation`: [Orientation]     - ориентация фрагмента
    /// - `&mut accumulator`: [ColorAccumulator] - указатель на структуру
    ///   (метод) накопления данных для дальнейшего определения результирующего
    ///   цвета
    fn process_chunk<A: ColorAccumulator>(
        &self, // чтобы сканер был многоразовым
        byte_frame: &[u8],
        chunk_start_index: usize,
        chunk_orientation: Orientation,
        accumulator: &mut A, // для последующего определения
    );
}

/// Реализация трейта для CheckerboardScanner
impl ChunkProcessor for CheckerboardScanner {
    fn process_chunk<A: ColorAccumulator>(
        &self,
        byte_frame: &[u8],
        chunk_start_index: usize,
        chunk_orientation: Orientation,
        accumulator: &mut A,
    ) {
        self.process_checkerboard_chunk(
            byte_frame,
            chunk_start_index,
            chunk_orientation,
            accumulator,
        );
    }
}
