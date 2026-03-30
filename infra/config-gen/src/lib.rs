pub use config_gen_macros::*;

// Скрываем зависимости, чтобы юзер их не ставил сам
pub mod __private {
    pub use serde;
    pub use toml;
}

pub trait TomlLoader: Sized {
    fn load_from_string(data: &str) -> Result<Self, String>;
}