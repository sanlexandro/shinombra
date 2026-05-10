//! Модуль обработки фрагмента
//! 
//! Необходим для правильного обхода фрагментов
//! 
//! Методы обработки фрагмента:
//! - В шахматном порядке [types::CheckerboardScanner] - данный метод проходит
//!   по фрагменту в жёстко фиксированном шахматном порядке. Дальнейшая работа с
//!   анализом цвета ведётся с помощью переданного метода обработки из
//!   [crate::analytics]


use crate::{analytics::ColorAnalyst, pixel_formatter::PixelFormatter, processing::processors::{configs::ChunkTask, types::CheckerboardScanner}};

pub mod checkerboard;
pub mod configs;
pub mod types;
pub mod registry;

pub use configs::Orientation;


/// Трейт обработки фрагмента
///
/// **Методы:**
/// - `process_chunk` - метод обработки фрагмента
pub trait ChunkProcessor<Formatter: PixelFormatter> {
    /// Обработка фрагмента
    ///
    /// Этот метод вызывается для обработки каждого фрагмента
    ///
    /// **Аргументы:**
    /// - `byte_frame`: &[[u8]]                  - указатель на кадр (массив
    ///   пикселей)
    /// - `chunk_task`: [ChunkTask]              - "задание" фрагмента
    /// - `&mut analyst`: [ColorAnalyst] - указатель на структуру
    ///   (метод) накопления данных для дальнейшего определения результирующего
    ///   цвета
    fn process_chunk<Analyst: ColorAnalyst>(
        &self, // чтобы сканер был многоразовым
        byte_frame: &[u8],
        chunk_task: ChunkTask,
        analyst: &mut Analyst, // для последующего определения
    );
}

/// Реализация трейта для CheckerboardScanner
impl<Formatter: PixelFormatter> ChunkProcessor<Formatter> for CheckerboardScanner {
    fn process_chunk<Analyst: ColorAnalyst>(
        &self,
        byte_frame: &[u8],
        chunk_task: ChunkTask,
        analyst: &mut Analyst,
    ) {
        self.process_checkerboard_chunk::<Formatter, Analyst>(
            byte_frame,
            chunk_task,
            analyst,
        );
    }
}
