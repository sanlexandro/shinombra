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
pub mod configs;
pub mod frame;
pub mod measures;
pub mod types;
pub mod configs_logic;
pub mod registry;

use super::processing::types::CheckerboardScanner;
use crate::analytics::ColorAccumulator;
use crate::processing::configs::ChunkTask;

/// Трейт обработки фрагмента
///
/// **Методы:**
/// - `process_chunk` - метод обработки фрагмента
pub trait ChunkProcessor {
    /// Обработка фрагмента
    ///
    /// Этот метод вызывается для обработки каждого фрагмента
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]]                  - указатель на кадр (массив
    ///   пикселей)
    /// - `chunk_task`: [ChunkTask]              - "задание" фрагмента
    /// - `&mut accumulator`: [ColorAccumulator] - указатель на структуру
    ///   (метод) накопления данных для дальнейшего определения результирующего
    ///   цвета
    fn process_chunk<A: ColorAccumulator>(
        &self, // чтобы сканер был многоразовым
        byte_frame: &[u8],
        chunk_task: ChunkTask,
        accumulator: &mut A, // для последующего определения
    );
}

/// Реализация трейта для CheckerboardScanner
impl ChunkProcessor for CheckerboardScanner {
    fn process_chunk<A: ColorAccumulator>(
        &self,
        byte_frame: &[u8],
        chunk_task: ChunkTask,
        accumulator: &mut A,
    ) {
        self.process_checkerboard_chunk(
            byte_frame,
            chunk_task,
            accumulator,
        );
    }
}
