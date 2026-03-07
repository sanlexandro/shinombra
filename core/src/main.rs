use algorithms::color_processor::color_processor::alg;
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

    while KEEP_RUNNING.load(Ordering::Relaxed) {
        capture.request_frame(|data| {
            // Теперь data — это безопасный &[u8]
            // Вызываем твой алгоритм из другого модуля
            alg(data, config.screen_width, config.screen_height);
        });
    }

    println!("\n\n\n\n---\n");
    println!("Width: {}", config.screen_width);
    println!("Hieght: {}", config.screen_height);

    println!("Shutting down...");

    capture.stop();
}
