use algorithms::{
    analytics::ColorAnalyst,
    filters::ColorFilter,
    pixel_formatter::PixelFormatter,
    processing::{processors::ChunkProcessor, types::ColorEngine},
};
use common::core::controller::CoreController;
use ffi::bindings::InitializingData;
use hardware_output::HardwareOutput;
use std::{ffi::CString, sync::Arc};
use threads::{
    hardware_output::hardware_output::HardwareOutputThread,
    screen_capture::screen_capture::CaptureThread,
};

pub mod bootstrap;
pub mod config;

use crate::bootstrap::ConfigLoader;

fn main() {
    // Инициализируем конфиг
    let config_loader = ConfigLoader::load();

    // Создаём контроллер
    let controller = Arc::new(CoreController::new(false));

    // Обработчик прерывания
    let controller_for_ctrlc = controller.clone();
    ctrlc::set_handler(move || {
        controller_for_ctrlc.shutdown();
    })
    .expect("Error setting Ctrl+C handler");

    let flags = config_loader.get_flags();

    let c_token = if !flags.session_token.is_empty() {
        Some(CString::new(flags.session_token.clone()).expect("Failed to create CString"))
    } else {
        None
    };

    // Берем указатель. Он будет валиден, пока жив c_token
    let token_ptr = c_token
        .as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null());

    // Запускаем инициализацию и `run_ambient_loop` в дальнейшем
    config_loader.run_stages(
        controller.clone(),
        InitializingData {
            apply_conversion: flags.pipewire_conversion,
            save_token: flags.save_token,
            token: token_ptr,
        },
    );

    println!("[INFO] Core: Shutting down...");
}

/// Запуск цикла обработки
///
/// Данная функция запускает основной цикл амбиентной подсветки, ответственный
/// за логику обмена данными между различными потоками и модулями
pub fn run_ambient_loop<Formatter, Processor, Analyst, Filter, Output>(
    mut color_engine: ColorEngine<Formatter, Processor, Analyst, Filter>,
    hardware_output: Output,
    led_amount: usize,
    capture_thread: &mut CaptureThread,
) where
    Formatter: PixelFormatter,
    Processor: ChunkProcessor<Formatter>,
    Analyst: ColorAnalyst,
    Filter: ColorFilter,
    Output: HardwareOutput + Send + 'static,
{
    capture_thread.calculate_data(Formatter::SIZE);

    let mut hardware_output_ctx =
        HardwareOutputThread::new(hardware_output, led_amount, capture_thread.get_controller());

    println!("[INFO] Core: The system is running!");

    let controller = capture_thread.get_controller();

    while controller.keep_running() {
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
