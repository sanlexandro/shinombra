use std::sync::atomic::{AtomicBool, Ordering};
use threads::screen_capture::screen_capture::{CaptureThread};
use ffi::bindings::{CaptureConfig};

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

    while KEEP_RUNNING.load(Ordering::Relaxed) {}

    println!("\n\n\n\n---\n");
    println!("Width: {}", config.screen_width);
    println!("Hieght: {}", config.screen_height);

    println!("Shutting down...");

    capture.stop();
}