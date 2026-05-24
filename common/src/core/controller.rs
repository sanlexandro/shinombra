//! Контроллер ядра

use std::sync::atomic::{AtomicBool, Ordering};

/// Контроллер ядра
///
/// Необходим для корректной обработки событий из разных потоков
///
/// **Поля:**
/// - `keep_running`: [AtomicBool] - флаг продолжения работы основного потока
pub struct CoreController {
    pub(super) keep_running: AtomicBool,
}

impl CoreController {
    /// Конструктор
    ///
    /// **Аргументы:**
    /// - `val`: [bool] - начальное состояние флага `keep_running`
    pub fn new(val: bool) -> Self {
        Self {
            keep_running: AtomicBool::new(val),
        }
    }

    /// Получение значения флага `keep_running`
    pub fn keep_running(&self) -> bool {
        self.keep_running.load(Ordering::Relaxed)
    }

    /// Запуск основного потока (`keep_running = true`)
    pub fn start(&self) {
        self.keep_running.store(true, Ordering::SeqCst);
    }

    /// Остановка основного потока (`keep_running = false`)
    pub fn shutdown(&self) {
        self.keep_running.store(false, Ordering::SeqCst);
    }
}
