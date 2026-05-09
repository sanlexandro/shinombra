use algorithms::{
    analytics::ColorAnalyst, filters::ColorFilter, pixel_formatter::PixelFormatter,
    processing::processors::ChunkProcessor,
};
use ffi::bindings::{CaptureConfig, CaptureEvent};
use hardware_output::HardwareOutput;
use std::{
    ffi::CStr,
    sync::atomic::{AtomicBool, Ordering},
};
use threads::{
    hardware_output::hardware_output::HardwareOutputThread,
    screen_capture::screen_capture::CaptureThread,
};

pub mod bootstrap;
pub mod config;

use crate::bootstrap::ConfigLoader;
use algorithms::processing::types::ColorEngine;

static KEEP_RUNNING: AtomicBool = AtomicBool::new(false);

// ф-я остановки основного потока
fn ctrlc_func() {
    KEEP_RUNNING.store(false, Ordering::SeqCst);
}

///
extern "C" fn on_capture_event(event: CaptureEvent, msg_ptr: *const std::os::raw::c_char) {
    let msg = unsafe { CStr::from_ptr(msg_ptr).to_string_lossy() };

    match event {
        CaptureEvent::Ready => {
            println!("[INFO] ScreenCapture: Streaming started ({})", msg);
            KEEP_RUNNING.store(true, Ordering::SeqCst);
        }
        CaptureEvent::Error => {
            eprintln!("[ERROR] ScreenCapture: Capture error: {}", msg);
            KEEP_RUNNING.store(false, Ordering::SeqCst);
        }
        CaptureEvent::Stopped => {
            println!("[INFO] ScreenCapture: Streaming stopped: {}", msg);
            KEEP_RUNNING.store(false, Ordering::SeqCst);
        }
        CaptureEvent::Reconnecting=> {
            println!("[INFO] ScreenCapture: Streaming reconnecting: {}", msg);
            KEEP_RUNNING.store(false, Ordering::SeqCst);
        }
    }
}

fn main() {
    // Обработчик прерывания
    ctrlc::set_handler(ctrlc_func).expect("Some errors!");

    // Инициализируем конфиг
    let config_loader = ConfigLoader::load();

    // Если конфиг прочитался корректно, запускаем поток захвата
    let mut capture_config = CaptureConfig::new();

    let mut capture_thread = CaptureThread::new();
    capture_thread.start(
        &mut capture_config,
        on_capture_event,
        config_loader.get_flags().pipewire_conversion,
    );

    // Ждем, пока флаг станет TRUE
    while !KEEP_RUNNING.load(Ordering::Relaxed) {
        // Спим 10мс, чтобы не грузить CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Запускаем инициализацию и `run_ambient_loop` в дальнейшем
    config_loader.run_stages(capture_config, &mut capture_thread);

    println!("[INFO] Core: Shutting down...");

    capture_thread.stop();
}

/// Запуск цикла обработки
///
/// Данная функция запускает основной цикл амбиентной подсветки, ответственный
/// за логику обмена данными между различными потоками и модулями
pub fn run_ambient_loop<Formatter, Processor, Analyst, Filter, Output>(
    mut color_engine: ColorEngine<Formatter, Processor, Analyst, Filter>,
    hardware_output: Output,
    led_amount: usize,
    capture_thread: &CaptureThread,
) where
    Formatter: PixelFormatter,
    Processor: ChunkProcessor<Formatter>,
    Analyst: ColorAnalyst,
    Filter: ColorFilter,
    Output: HardwareOutput + Send + 'static,
{
    let mut hardware_output_ctx = HardwareOutputThread::new(hardware_output, led_amount);

    println!("[INFO] Core: The system is running!");

    while KEEP_RUNNING.load(Ordering::Relaxed) {
        // Запрашиваем кадр
        capture_thread.request_frame(|data| {
            // Быстро читаем и анализируем
            color_engine.process_frame(data);
        });

        // Применяем фильтры
        let colors = color_engine.apply_filters();

        // Отправляем на устройство
        hardware_output_ctx.update_colors(colors);
    }

    hardware_output_ctx.stop();
}
