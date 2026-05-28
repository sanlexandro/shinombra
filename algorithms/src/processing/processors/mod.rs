//! Модуль обработки фрагмента
//!
//! Необходим для правильного обхода фрагментов
//!
//! Методы обработки фрагмента:
//! - В шахматном порядке [types::CheckerboardScanner] - данный метод проходит
//!   по фрагменту в жёстко фиксированном шахматном порядке. Дальнейшая работа с
//!   анализом цвета ведётся с помощью переданного метода обработки из
//!   [crate::analytics]

use crate::{
    analytics::ColorAnalyst, pixel_formatter::PixelFormatter,
    processing::processors::configs::ChunkTask,
};

pub mod checkerboard;
pub mod configs;
pub mod dynamic_checkerboard;
pub mod registry;
pub mod states;
pub mod types;

pub use configs::Orientation;

/// Трейт обработки фрагмента
///
/// **Методы:**
/// - `process_chunk` - метод обработки фрагмента
pub trait ChunkProcessor<Formatter: PixelFormatter, Analyst: ColorAnalyst> {
    /// Соответствующая структура хранения состояния
    type State: ChunkState;

    /// Обработка фрагмента
    ///
    /// Этот метод вызывается для обработки каждого фрагмента
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]]            - указатель на кадр (массив пикселей)
    /// - `chunk_task`: &[ChunkTask]       - "задание" фрагмента
    /// - `chunk_state`: &mut [ChunkState] - пустое состояние фрагмента
    /// - `&mut analyst`: [ColorAnalyst]   - указатель на структуру
    ///   (метод) накопления данных для дальнейшего определения результирующего
    ///   цвета
    fn process_chunk(
        &self, // чтобы сканер был многоразовым
        byte_frame: &[u8],
        chunk_task: &ChunkTask,
        chunk_state: &mut Self::State,
        analyst: &mut Analyst, // для последующего определения
    );
}

/// Трейт состояния фрагмента
///
/// Необходим для выделения необходимых областей памяти для хранения контекстов
/// соответствующих алгоритмов
pub trait ChunkState: Copy {
    fn new() -> Self
    where
        Self: Sized;
}
