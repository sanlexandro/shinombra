/// Уровни ошибок
#[derive(Copy, Clone, PartialEq)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}