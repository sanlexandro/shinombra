//! Обработка cli флагов

use std::{path::PathBuf, process::exit};

use common::names::*;
use logger::*;

const MODULE: &str = "CLIFlagsManager";

/// Все возможные cli флаги
///
/// **Поля:**
/// `manifest_path`: [PathBuf]               - путь до манифеста
/// `reset_pipewire_token`: [bool]           - сброс pipewire-токена
/// `max_log_level`: [Option]<[LogLevel]>    - максимальный уровень логов
pub struct CLIFlags {
    pub manifest_path: PathBuf,
    pub reset_pipewire_token: bool,
    pub max_log_level: Option<LogLevel>,
}

pub struct CLIFlagsManager;

impl CLIFlagsManager {
    /// Парсинг cli флагов
    pub fn parse() -> CLIFlags {
        let mut args = std::env::args().skip(1);

        // Дефолтные значения
        let mut manifest_path = PathBuf::from("config.toml");
        let mut reset_pipewire_token = false;
        let mut max_log_level = Option::<LogLevel>::None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-p" | "--path" => {
                    if let Some(next_arg) = args.next() {
                        manifest_path = PathBuf::from(next_arg);
                    } else {
                        error!("-p/--path requires a valid path argument.");
                        exit(1);
                    }
                }
                "-r" | "--reset" => {
                    reset_pipewire_token = true;
                }
                "-v" | "--verbose" | "--debug" => {
                    max_log_level = Some(LogLevel::Debug);
                }
                "-V" | "--version" => {
                    println!(
                        "{} v{}\nauthor: {}",
                        APP_NAME,
                        env!("CARGO_PKG_VERSION"),
                        AUTHOR_NAME
                    );
                    exit(0);
                }
                "-h" | "--help" => {
                    Self::print_help();
                    exit(0);
                }
                unknown => {
                    warn!("Unknown argument '{}' ignored.", unknown);
                }
            }
        }

        CLIFlags {
            manifest_path,
            reset_pipewire_token,
            max_log_level
        }
    }

    pub fn print_help() {
        let help = include_str!("../../docs/help.txt");
        print!("{}", help);
    }
}
