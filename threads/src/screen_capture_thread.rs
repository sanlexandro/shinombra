//! Модуль, управляющий потоком захвата экрана

use std::os::raw::c_void;
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;

#[repr(C)]
pub struct CaptureConfig {
    pub screen_width: u32,    // Заполняет C (PipeWire)
    pub screen_height: u32,   // Заполняет C (PipeWire)
    pub is_ready: AtomicBool, // Флаг для синхронизации
}

#[repr(C)]
pub struct CaptureThread {
    handle: Option<JoinHandle<()>>,
    ctx_ptr: *mut c_void,
}

// Используем ф-ии из `C-worker`
extern "C" {
    /// Ф-я инициализации захвата экрана
    fn screen_capture_init(config: *mut CaptureConfig) -> *mut c_void;

    /// Ф-я запуска захвата экрана
    fn screen_capture_run(ctx: *mut c_void);

    /// Ф-я остановки потока захвата экрана
    fn screen_capture_stop(ctx:*mut c_void);
}

impl CaptureThread {
    /// Ф-я инициализации потока
    pub fn new() -> Self {
        return CaptureThread {
            handle: None,
            ctx_ptr: std::ptr::null_mut(),
        };
    }

    /// Ф-я запуска потока
    pub fn start(&mut self, config: &mut CaptureConfig) {
        // Проверяем, не запущен ли уже поток, чтобы не плодить их
        if self.handle.is_some() {
            println!("[WARN] Thread already running");
            return;
        }

        let ctx: *mut std::ffi::c_void = unsafe {
            screen_capture_init(config as *mut CaptureConfig)
        };

        if ctx.is_null() {
            println!("[ERROR] Failed to initialize C context");
            return;
        }

        let ctx_for_thread = ctx as usize;

        // Запускаем поток
        let thread_handle = std::thread::spawn(move || {
            let ptr = ctx_for_thread as *mut c_void;
            unsafe {
                screen_capture_run(ptr);
            }
        });

        self.handle = Some(thread_handle);
    }

    /// Ф-я остановки потока
    pub fn stop(&mut self) {
        if self.ctx_ptr.is_null() {
            println!("[WARN] No context to stop");
            return;
        }

        unsafe {
            // Передаем указатель в C, чтобы вызвать pw_main_loop_quit
            screen_capture_stop(self.ctx_ptr);
        }

        if let Some(handle) = self.handle.take() {
            handle.join().expect("Couldn't join thread");
            self.ctx_ptr = std::ptr::null_mut(); // Обнуляем после завершения
            println!("[INFO] Capture thread stopped");
        }
    }
}
