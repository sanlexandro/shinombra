//! Модуль, управляющий потоком захвата экрана

use std::thread::JoinHandle;

// Используем ф-ии из `C-worker`
extern "C" {
    /// Ф-я захвата экрана
    fn screen_capture_init() -> i32;

    /// Ф-я остановки потока захвата экрана
    fn screen_capture_stop_thread();
}

#[repr(C)]
pub struct CaptureConfig {
    pub screen_width: u32,  // Заполняет C (PipeWire)
    pub screen_height: u32, // Заполняет C (PipeWire)
    pub capture_depth: u32, // Заполняет Rust (из UI/Config)
    pub capture_step: u32,  // Заполняет Rust (из UI/Config)
    pub is_ready: bool,     // Флаг для синхронизации
}


pub struct CaptureThread {
    handle: Option<JoinHandle<()>>,
}

impl CaptureThread {
    /// Ф-я инициализации потока
    pub fn new() -> Self {
        return CaptureThread { handle: None };
    }

    /// Ф-я запуска потока
    pub fn start(&mut self) {
        // Проверяем, не запущен ли уже поток, чтобы не плодить их
        if self.handle.is_some() {
            println!("[WARN] Thread already running");
            return;
        }

        // Запускаем поток
        let thread_handle = std::thread::spawn(|| unsafe {
            screen_capture_init();
        });

        // Сохраняем в "себя"
        self.handle = Some(thread_handle);
    }

    /// Ф-я остановки потока
    pub fn stop(&mut self) {
        // Останавливаем поток захвата
        unsafe {
            screen_capture_stop_thread();
        }

        // Завершаем поток с проверкой
        if let Some(handle) = self.handle.take() {
            handle.join().expect("Couldn't join thread");
            println!("[INFO] Capture thread stopped");
        } else {
            println!("[WARN] No thread to stop");
        }
    }
}
