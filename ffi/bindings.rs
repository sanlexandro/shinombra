//! Подключение ф-ий из C

use std::os::raw::{c_void};
use std::sync::atomic::{AtomicBool};

#[repr(C)]
pub struct CaptureConfig {
    pub screen_width: u32,    // Заполняет C (PipeWire)
    pub screen_height: u32,   // Заполняет C (PipeWire)
    pub is_ready: AtomicBool, // Флаг для синхронизации
}

// Используем ф-ии из `C-worker`
extern "C" {
    /// Ф-я инициализации захвата экрана
    pub fn screen_capture_init(config: *mut CaptureConfig) -> *mut c_void;

    /// Ф-я запуска захвата экрана
    pub fn screen_capture_run(ctx: *mut c_void);

    /// Ф-я остановки потока захвата экрана
    pub fn screen_capture_stop(ctx:*mut c_void);

    /// Ф-я получения указателя DMA
    pub fn wait_for_frame(ctx:*mut c_void) -> *mut u8;

    /// Ф-я освобождения кадра DMA
    pub fn release_frame(ctx:*mut c_void);
}