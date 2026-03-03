//! Описание компиляции C-файла, необходимого для работы потока

use std::path::PathBuf;

fn main() {
    // Подготавливаем пути к папкам
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let mut c_file_path = PathBuf::from(manifest_dir);
    c_file_path.push("../c-worker/src/pw-screen-capture.c");

    // Говорим пересобирать проект при изменении файла
    println!("cargo:rerun-if-changed={}", c_file_path.display());

    // Список всех нужных системных библиотек
    let libs = ["libpipewire-0.3", "libportal", "gio-2.0"];

    // Запускаем сборку файла
    let mut build = cc::Build::new();
    build.file(c_file_path);

    for lib in libs {
        let library =
            pkg_config::probe_library(lib).unwrap_or_else(|_| panic!("Lib {} didn`t found!", lib));

        // Добавляем пути к инклудам каждой найденной либы
        for path in library.include_paths {
            build.include(path);
        }
    }

    // Завершаем
    build.compile("capture_worker");
}
