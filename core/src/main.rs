use std::sync::atomic::{AtomicBool, Ordering};
use threads::screen_capture_thread::CaptureThread;

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

    let mut capture = CaptureThread::new();

    capture.start();

    while KEEP_RUNNING.load(Ordering::Relaxed) {}

    println!("Shutting down...");

    capture.stop();
}