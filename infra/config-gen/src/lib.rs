pub use config_gen_macros::*;

// Скрываем зависимости, чтобы юзер их не ставил сам
pub mod __private {
    pub use serde;
    pub use toml;
}

/// Трейт загрузки данных из toml-конфига
pub trait TomlLoader: Sized {
    /// Метод загрузки строки
    fn load_from_string(data: &str) -> Result<Self, String>;
}