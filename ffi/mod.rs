pub mod bindings;

// Доверяем C коду (его контекст можно передавать между потоками)
unsafe impl Send for bindings::CaptureConfig {}
unsafe impl Sync for bindings::CaptureConfig {}
