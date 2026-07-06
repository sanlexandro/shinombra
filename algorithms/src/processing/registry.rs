//! Различные настройки для рамки

/// Варианты крепления рамок
#[derive(Clone, Copy, PartialEq)]
pub enum FrameElement {
    Left,
    Up,
    Right,
    Down,
}

/// Направление для UsualFrame (всего кадра по кругу)
#[derive(Clone, Copy)]
pub enum ClockDirection {
    Clockwise,
    Counterclockwise,
}
