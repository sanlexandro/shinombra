use algorithms::{
    analytics::types::ColorHistogram,
    processing::{
        configs::{
            CheckerboardConfig, GeometryConfig, LedPositionConfig, ScreenConfig,
            ScreenReadingConfig,
        },
        types::{CheckerboardScanner, ColorEngine},
    },
    units::{Millimeters, Pixels},
};
use ffi::bindings::CaptureConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use threads::screen_capture::screen_capture::CaptureThread;

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

    let mut config = CaptureConfig {
        screen_width: 0,
        screen_height: 0,
        is_ready: AtomicBool::new(false),
    };

    let mut capture = CaptureThread::new();

    capture.start(&mut config);

    // Ждем, пока флаг станет TRUE
    while !config.is_ready.load(Ordering::Relaxed) {
        // Спим 10мс, чтобы не грузить CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Заполняем конфигурацию для обработки цвета
    let screen_config = ScreenConfig {
        frame_width_px: Pixels(config.screen_width as usize),
        frame_height_px: Pixels(config.screen_height as usize),

        frame_width_mm: Millimeters(595),
        frame_height_mm: Millimeters(335),
    };

    let geometry = GeometryConfig {
        led_pos: LedPositionConfig {
            gap: Millimeters(5),
            led_length: Millimeters(62),

            horizontal_offset: Millimeters(18),
            horizontal_led_amount: 9,

            vertical_offset: Millimeters(12),
            vertical_led_amount: 5,
        },
        reading: ScreenReadingConfig {
            deep_in: Millimeters(20),
            deep_out: Millimeters(0),
        },
    };

    let alg_config = CheckerboardConfig {
        config: geometry.calculate_chunk_config(screen_config),
        pixel_step: 5,
        row_stride: 3,
    };

    let processor = CheckerboardScanner::new(alg_config, screen_config);

    let accumulator = ColorHistogram::new();

    let mut color_engine = ColorEngine::new(processor, accumulator, geometry, screen_config);

    while KEEP_RUNNING.load(Ordering::Relaxed) {
        capture.request_frame(|data| {
            // Теперь data — это безопасный &[u8]
            color_engine.process_frame(data);
        });
    }

    println!("\n\n\n\n---\n");
    println!("Width: {}", config.screen_width);
    println!("Height: {}", config.screen_height);

    println!("Shutting down...");

    capture.stop();
}
