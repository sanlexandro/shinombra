/// Тип ориентации фрагмента
// #[derive(PartialEq, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Кортежная структура для работы с миллиметрами
// #[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub struct Millimeters(pub u32);

/// Кортежная структура для работы с пикселями
// #[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub struct Pixels(pub usize);
