//! Объявление различных вариантов аналитики цвета

#[derive(Debug)]
pub enum ColorAnalystType {
    Histogram,
    Average,
    DebugRGB,
}
