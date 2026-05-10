/// Кортежная структура для работы с миллиметрами
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Millimeters(pub u32);

/// Кортежная структура для работы с пикселями
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Pixels(pub usize);
