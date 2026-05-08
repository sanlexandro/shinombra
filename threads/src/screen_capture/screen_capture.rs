//! Модуль, управляющий потоком захвата экрана

use ffi::bindings::*;
use hardware_output::debug;
use std::ffi::c_void;
use std::thread::JoinHandle;

/// Поток захвата кадра
///
/// **Поля:**
/// `handle`: [Option]<JoinHandle<()>> - поток
/// `ctx_ptr`: *mut [c_void] - контекст потока
/// `frame_size`: [usize] - размер кадра
pub struct CaptureThread {
    pub handle: Option<JoinHandle<()>>,
    pub ctx_ptr: *mut c_void,
    pub frame_size: usize,
}

impl CaptureThread {
    /// Ф-я инициализации потока
    pub fn new() -> Self {
        return CaptureThread {
            handle: None,
            ctx_ptr: std::ptr::null_mut(),
            frame_size: 0,
        };
    }

    /// Ф-я запуска потока
    pub fn start(&mut self, config: &mut CaptureConfig) {
        // Проверяем, не запущен ли уже поток, чтобы не плодить их
        if self.handle.is_some() {
            println!("[WARN] Thread already running");
            return;
        }

        // Запускаем поток захвата и сохраняем контекст и данные об экране
        let ctx: *mut std::ffi::c_void =
            unsafe { screen_capture_init(config as *mut CaptureConfig) };

        // Проверяем, что получили не нулевой контекст
        if ctx.is_null() {
            println!("[ERROR] Failed to initialize C context");
            return;
        }

        // Сохраняем контекст
        self.ctx_ptr = ctx;
        
        // Запускаем поток
        let ctx_for_thread = ctx as usize;
        let thread_handle = std::thread::spawn(move || {
            let ptr = ctx_for_thread as *mut c_void;
            unsafe {
                screen_capture_run(ptr);
            }
        });

        self.handle = Some(thread_handle);
    }

    pub fn calculate_data(&mut self, config: &CaptureConfig) {
        // test
        println!("video format: {}", SpaVideoFormat::try_from(config.video_format).map(|f| f.to_string()).unwrap_or_else(|e| format!("unknown ({})", e)));

        // Рассчитываем размер кадра
        self.frame_size = (config.screen_height * config.screen_width * 4) as usize; // TODO: считывание размера одного пикселя
    }

    /// Остановка потока
    pub fn stop(&mut self) {
        if self.ctx_ptr.is_null() {
            println!("[WARN] No context to stop");
            return;
        }

        unsafe {
            // Передаем указатель в C, чтобы вызвать `pw_main_loop_quit`
            screen_capture_stop(self.ctx_ptr);
        }

        if let Some(handle) = self.handle.take() {
            handle.join().expect("Couldn't join thread");
            self.ctx_ptr = std::ptr::null_mut(); // Обнуляем после завершения
            println!("[INFO] Capture thread stopped");
        }
    }

    /// Запрос кадра и его обработка
    ///
    /// Обёртка над запросом, которая вызывает переданную ф-ю для обработки кадра
    ///
    /// **Входные поля:**
    /// - `processor`: [FnOnce] (&[[u8]]) - метод обработки кадра соответствующей
    ///   реализации [algorithms::processing::ChunkProcessor]
    pub fn request_frame<F>(&self, processor: F)
    where
        F: FnOnce(&[u8]),
    {
        unsafe {
            if self.ctx_ptr.is_null() {
                return;
            }

            // Ждем, пока Си-воркер подготовит кадр (блокирующий вызов)
            let ptr = wait_for_frame(self.ctx_ptr);

            if !ptr.is_null() {
                // Создаем слайс (окно в память Си)
                let data = std::slice::from_raw_parts(ptr, self.frame_size);

                // Выполняем полученный алгоритм
                processor(data);

                // Обязательно сообщаем Си, что мы закончили работать с этим буфером
                release_frame(self.ctx_ptr);
            }
        }
    }
}
