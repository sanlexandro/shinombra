//! Общие функции для работы с путями

use std::path::{Path, PathBuf};

/// Разворачивает `~` в домашний каталог пользователя.
///
/// Если домашний каталог не удалось определить, путь возвращается без изменений.
pub fn expand_tilde(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    let home = std::env::var_os("HOME").map(PathBuf::from);

    if let Some(home) = home {
        if path_str == "~" {
            return home;
        }

        if path_str.starts_with("~/") {
            return home.join(path_str.trim_start_matches("~/"));
        }
    }

    path.to_path_buf()
}

/// Разрешает путь относительно базовой директории.
///
/// Сначала раскрывает `~`, затем, если путь относительный, приклеивает его к `base_dir`.
pub fn resolve_path(base_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> PathBuf {
    let expanded = expand_tilde(path);

    if expanded.is_relative() {
        base_dir.as_ref().join(expanded)
    } else {
        expanded
    }
}
