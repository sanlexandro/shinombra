/// Тип ориентации фрагмента
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Кортежная структура для работы с миллиметрами
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Millimeters(pub u32);

/// Кортежная структура для работы с пикселями
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Pixels(pub usize);
