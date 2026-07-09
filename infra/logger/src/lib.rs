//! Простой логгер

pub use crate::registry::LogLevel;
pub mod registry;
use std::sync::atomic::{AtomicU8, Ordering};

// Глобальная переменная уровня логов (по умолчанию WARN)
pub static MAX_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Warn as u8);

pub fn set_max_log_level(level: LogLevel) {
    MAX_LOG_LEVEL.store(level as u8, Ordering::SeqCst);
}

#[macro_export]
macro_rules! log {
    ($level:expr, $module:expr, $($arg:tt)*) => {{
        let msg_level = $level as u8; // Приводим enum к числу

        // Сравниваем с глобальным порогом
        if msg_level <= $crate::MAX_LOG_LEVEL.load(std::sync::atomic::Ordering::Relaxed) {
            let (prefix, color, label) = match $level {
                LogLevel::Error => ("<3>", "\x1b[31m", "ERROR"),
                LogLevel::Warn  => ("<4>", "\x1b[33m", "WARN "),
                LogLevel::Info  => ("<6>", "\x1b[32m", "INFO "),
                LogLevel::Debug => ("<7>", "\x1b[36m", "DEBUG"),
            };

            eprintln!(
                "{}{} [{}] \x1b[0m{}: {}",
                prefix, color, label, $module, format_args!($($arg)*)
            );
        }
    }};
}

// Удобные обертки для отображения
#[macro_export]
macro_rules! info  { ($($arg:tt)*) => { log!(LogLevel::Info, MODULE,  $($arg)*) }; }
#[macro_export]
macro_rules! error { ($($arg:tt)*) => { log!(LogLevel::Error, MODULE, $($arg)*) }; }
#[macro_export]
macro_rules! warn  { ($($arg:tt)*) => { log!(LogLevel::Warn, MODULE,  $($arg)*) }; }
#[macro_export]
macro_rules! debug { ($($arg:tt)*) => { log!(LogLevel::Debug, MODULE, $($arg)*) }; }
