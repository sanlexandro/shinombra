use algorithms::{analytics::ColorAnalyst, filters::ColorFilter, processing::ChunkProcessor};
use ffi::bindings::CaptureConfig;
use hardware_output::HardwareOutput;
use std::sync::atomic::{AtomicBool, Ordering};
use threads::{
    hardware_output::hardware_output::HardwareOutputThread,
    screen_capture::screen_capture::CaptureThread,
};

pub mod bootstrap;
pub mod config;

use crate::bootstrap::ConfigLoader;
use algorithms::processing::types::ColorEngine;

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

// ф-я остановки основного потока
fn ctrlc_func() {
    println!("Got signal");
    KEEP_RUNNING.store(false, Ordering::Relaxed);
}

fn main() {
    println!("Ambient Lighting Backend");

    // Обработчик прерывания
    ctrlc::set_handler(ctrlc_func).expect("Some errors!");

    // Инициализируем конфиг
    let config_loader = ConfigLoader::load();

    // Если конфиг прочитался корректно, запускаем поток захвата
    let mut capture_config = CaptureConfig::new();

    let mut capture_thread = CaptureThread::new();
    capture_thread.start(&mut capture_config);

    // Ждем, пока флаг станет TRUE
    while !capture_config.is_ready.load(Ordering::Relaxed) {
        // Спим 10мс, чтобы не грузить CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Запускаем инициализацию и `run_ambient_loop` в дальнейшем
    config_loader.run_stages(
        capture_config.screen_width as u32,
        capture_config.screen_height as u32,
        &capture_thread,
    );

    println!("Shutting down...");

    capture_thread.stop();
}

/// Запуск цикла обработки
/// 
/// Данная функция запускает основной цикл амбиентной подсветки, ответственный
/// за логику обмена данными между различными потоками и модулями
pub fn run_ambient_loop<Processor, Analyst, Filter, Output>(
    mut color_engine: ColorEngine<Processor, Analyst, Filter>,
    hardware_output: Output,
    led_amount: usize,
    capture_thread: &CaptureThread,
) where
    Processor: ChunkProcessor,
    Analyst: ColorAnalyst,
    Filter: ColorFilter,
    Output: HardwareOutput + Send + 'static,
{
    let mut hardware_output_ctx = HardwareOutputThread::new(hardware_output, led_amount);

    println!("Система запущена!");

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
    println!("Цикл обработки завершен.");
}
