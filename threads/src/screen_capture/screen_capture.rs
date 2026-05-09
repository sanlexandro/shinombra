//! Модуль, управляющий потоком захвата экрана

use common::core::controller::CoreController;
use ffi::bindings::*;
use std::ffi::c_void;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::screen_capture::handler::callback;

/// Поток захвата кадра
///
/// **Поля:**
/// `handle`: [Option]<JoinHandle<()>> - поток
/// `ctx_ptr`: *mut [c_void] - контекст потока
/// `frame_size`: [usize] - размер кадра
/// `core_controller`: [Arc]<[CoreController]> - контроллер ядра
pub struct CaptureThread {
    pub(super) handle: Option<JoinHandle<()>>,
    pub(super) ctx_ptr: *mut c_void,
    pub(super) frame_size: usize,
    pub(super) core_controller: Arc<CoreController>,
}

impl CaptureThread {
    /// Запуск потока
    ///
    /// **Аргументы:**
    /// - `config`: &mut [CaptureConfig] - место, куда будет сохранён конфиг захвата
    /// - `core_controller`: [Arc]<[CoreController]> - контроллер ядра
    /// - `apply_conversion`: [bool] - флаг разрешения использовать
    ///   преобразования pipewire
    /// 
    /// **Выходные поля:**
    /// - [Result]<Self, [String]> - результат или текст ошибки
    pub fn new(
        config: &mut CaptureConfig,
        core_controller: Arc<CoreController>,
        apply_conversion: bool,
    ) -> Result<Self, String> {
        if apply_conversion {
            println!("[WARN] CaptureThread: PipeWire conversion applied")
        }

        // Запускаем поток захвата и сохраняем контекст и данные об экране
        let user_data = Arc::into_raw(core_controller.clone()) as *mut c_void;
        let ctx: *mut std::ffi::c_void = unsafe {
            screen_capture_init(
                config as *mut CaptureConfig,
                callback,
                user_data,
                apply_conversion,
            )
        };

        // Проверяем, что получили не нулевой контекст
        if ctx.is_null() {
            unsafe { Arc::from_raw(user_data as *const CoreController); } // Вернули и дропнули
            println!("[ERROR] CaptureThread: Failed to initialize C context");
            return Err("Failed to initialize C context".to_string());
        }

        // Запускаем поток
        let ctx_for_thread = ctx as usize;
        let thread_handle = std::thread::spawn(move || {
            let ptr = ctx_for_thread as *mut c_void;
            unsafe {
                screen_capture_run(ptr);
            }
        });

        return Ok(Self {
            handle: Some(thread_handle),
            ctx_ptr: ctx,
            frame_size: 0,
            core_controller,
        });
    }

    /// Получение контроллера ядра
    pub fn get_controller(&self) -> Arc<CoreController> {
        self.core_controller.clone()
    }

    /// Расчёт необходимых для работы данных
    ///
    /// **Аргументы:**
    /// - `pixel_size`: [usize] - размер одного пикселя в байтах
    ///
    pub fn calculate_data(&mut self, pixel_size: usize) {
        // Получаем указатель
        let config_ptr: *mut CaptureConfig = unsafe { get_capture_config(self.ctx_ptr) };

        if config_ptr.is_null() {
            eprintln!("[ERROR] CaptureThread: Config pointer is NULL");
            return;
        }

        // Превращаем указатель в безопасную ссылку (разыменовываем внутри unsafe)
        let config = unsafe { &*config_ptr };

        let format_str = SpaVideoFormat::try_from(config.video_format)
            .map(|f| f.to_string())
            .unwrap_or_else(|_| format!("unknown ({})", config.video_format));

        println!("[INFO] CaptureThread: video format: {}", format_str);

        // Рассчитываем размер кадра
        self.frame_size = (config.screen_height * config.screen_width) as usize * pixel_size;
    }

    /// Остановка потока
    pub fn stop(&mut self) {
        if self.ctx_ptr.is_null() {
            println!("[WARN] CaptureThread: No context to stop");
            return;
        }

        unsafe {
            // Передаем указатель в C, чтобы вызвать `pw_main_loop_quit`
            screen_capture_stop(self.ctx_ptr);
        }

        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .expect("[ERROR] CaptureThread: Couldn't join thread");
            self.ctx_ptr = std::ptr::null_mut(); // Обнуляем после завершения
            println!("[INFO] CaptureThread: Capture thread stopped");
        }
    }

    /// Запрос кадра и его обработка
    ///
    /// Обёртка над запросом, которая вызывает переданную ф-ю для обработки кадра
    ///
    /// **Аргументы:**
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
