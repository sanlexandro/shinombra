#[repr(C)]
pub struct CaptureConfig {
    pub screen_width: u32,  // Заполняет C (PipeWire)
    pub screen_height: u32, // Заполняет C (PipeWire)
    pub capture_depth: u32, // Заполняет Rust (из UI/Config)
    pub capture_step: u32,  // Заполняет Rust (из UI/Config)
    pub is_ready: bool,     // Флаг для синхронизации
}

fn main() {
    println!("Ambient Lighting Backend");
}
