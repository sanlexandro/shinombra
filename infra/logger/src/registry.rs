/// Уровни ошибок
#[derive(Copy, Clone, PartialEq)]
pub enum LogLevel {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}
