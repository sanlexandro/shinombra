pub use config_gen_macros::*;

// Скрываем зависимости
pub mod __private {
    pub use serde;
    pub use toml;
}