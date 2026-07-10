//! Модуль загрузки данных из конфига и подготовки структур для запуска цикла
//!
//! Данный модуль вытягивает данные из toml-файла и запускает сборку данных по ступеням:
//! 1) выбор анализатора цвета в фрагменте
//! 2) выбор фильтра для результирующего цвета
//! 3) запуск потока захвата
//! 4) выбор преобразователя пикселей
//! 5) выбор обработчика фрагментов
//! 6) сборка движка и выбор вывода на устройство
//!
//! После подготовки всех ступеней запускается основной цикл программы,
//! содержащийся в main

const MODULE: &str = "ConfigLoader";

use crate::cli_flags::CLIFlags;
use crate::config::*;
use crate::run_ambient_loop;
use algorithms::{
    analytics::{configs::*, registry::*, types::*, ColorAnalyst},
    filters::{configs::*, registry::*, types::*, ColorFilter},
    pixel_formatter::{types::*, PixelFormatter},
    processing::{
        configs::*,
        processors::{configs::*, registry::*, types::*, ChunkProcessor, Orientation},
        registry::*,
        types::*,
    },
};
use common::crypto::xor_crypt;
use common::names::*;
use common::{configs::*, core::controller::CoreController, units::*};
use config_gen::{__private::*, *};
use ffi::bindings::{CaptureConfig, InitializingData, SpaVideoFormat};
use hardware_output::{
    debug::types::DebugDriver,
    registry::*,
    serial::{config::SerialDriverConfig, types::SerialDriver},
};
use logger::*;
use std::{ffi::CString, path::PathBuf, process::exit, sync::Arc};
use threads::screen_capture::screen_capture::CaptureThread;

include_shadow_all!(
    "./common/src/units/units.rs",
    "./algorithms/src/processing/registry.rs",
    "./algorithms/src/processing/configs/configs.rs",
    "./algorithms/src/processing/processors/registry.rs",
    "./algorithms/src/processing/processors/configs/configs.rs",
    "./algorithms/src/analytics/registry.rs",
    "./algorithms/src/analytics/configs/configs.rs",
    "./algorithms/src/filters/registry.rs",
    "./algorithms/src/filters/configs/configs.rs",
    "./hardware_output/src/registry.rs",
    "./core/src/config.rs",
    "./hardware_output/src/serial/config/config.rs",
    "./infra/logger/src/registry.rs",
);

pub struct ConfigLoader {
    pub settings: Settings,
    pub daemon_settings: Option<DaemonSettings>,
    pub shadow_root: FullConfigShadow, // Храним временно для инициализации алгоритмов
    pub geometry_config: GeometryConfig,
    pub screen_config: ScreenConfig,
    pub cli_flags: CLIFlags,
}

impl ConfigLoader {
    /// Вспомогательная ф-я для склеивания toml-таблиц
    ///
    /// Необходима для корректной работы с overlays
    pub(super) fn merge_toml_tables(base: &mut toml::Table, overlay: toml::Table) {
        for (key, value) in overlay {
            match value {
                // Если внутри оверлея лежит подтаблица (например, [settings])
                toml::Value::Table(overlay_sub_table) => {
                    // Проверяем, есть ли такая таблица в базе
                    if let Some(toml::Value::Table(base_sub_table)) = base.get_mut(&key) {
                        // Если есть, рекурсивно мержим их внутренности
                        ConfigLoader::merge_toml_tables(base_sub_table, overlay_sub_table);
                    } else {
                        // Если в базе такой таблицы не было вообще, просто вставляем целиком
                        base.insert(key, toml::Value::Table(overlay_sub_table));
                    }
                }
                // Для всех остальных типов данных (строки, числа, массивы) — оверлей просто затирает базу
                _ => {
                    base.insert(key, value);
                }
            }
        }
    }

    /// Определение положения файла сессии
    pub(super) fn get_session_path() -> PathBuf {
        // Пытаемся получить путь к $HOME/.local/share
        let mut path = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local").join("share")
        } else {
            // Запасной вариант на случай странного окружения
            PathBuf::from(".")
        };
        path.push(APP_NAME);

        // Создаем папку, если её ещё нет (mkdir -p)
        if let Err(e) = std::fs::create_dir_all(&path) {
            warn!("Failed to create session directory {:?}: {}", path, e);
        }

        // Имя самого файла сессии
        path.join(SESSION_NAME)
    }

    /// Инициализация конфигурации
    ///
    /// Данная функция пытается:
    /// 1) прочитать данные из манифеста (пропускается в простом режиме)
    /// 2) прочитать данные из файла конфигурации
    /// 3) наложить на них данные из overlays
    /// 4) извлечь критически важные настройки
    /// 5) проверить эти настройки
    ///
    /// Далее она преобразует их в готовые для работы данные, которые необходимы
    /// для запуска дальнейшей инициализации
    pub fn load(cli_flags: CLIFlags) -> Self {
        // итоговые настройки демона
        let mut daemon_settings = Option::<DaemonSettings>::None;

        // Если был передан путь до конфигурации, запускаем простой режим, минуя
        // манифесты и оверлеи
        // Получаем финальную таблицу
        let final_table = match cli_flags.config_path.as_ref() {
            Some(config_path) => {
                let config_text = match std::fs::read_to_string(config_path) {
                    Ok(text) => text,
                    Err(e) => {
                        error!(
                            "Critical failure during loading config {}: {}",
                            config_path.to_string_lossy(),
                            e
                        );
                        exit(1);
                    }
                };
                let final_table = match toml::from_str(&config_text) {
                    Ok(table) => table,
                    Err(e) => {
                        println!(
                            "Critical failure during mapping result toml to shadow struct: {}",
                            e
                        );
                        exit(1);
                    }
                };
                final_table
            }
            None => {
                // Считываем данные из манифеста
                let manifest_str = match std::fs::read_to_string(&cli_flags.manifest_path) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        error!(
                            "Critical failure during loading manifest {} : {}",
                            cli_flags.manifest_path.to_string_lossy(),
                            e
                        );
                        exit(1);
                    }
                };

                // Проводим десериализцию
                let shadow_manifest: ManifestShadow = match toml::from_str(&manifest_str) {
                    Ok(root) => root,
                    Err(e) => {
                        error!(
                            "Critical failure during reading manifest toml {} : {}",
                            manifest_str, e
                        );
                        exit(1);
                    }
                };
                let manifest: Manifest = shadow_manifest.into();

                // Получаем абсолютный путь к самому манифесту
                let manifest_path = std::path::Path::new(&cli_flags.manifest_path)
                    .canonicalize()
                    .unwrap_or_else(|_| std::path::PathBuf::from(&cli_flags.manifest_path));
                let manifest_dir = manifest_path.parent().unwrap_or(std::path::Path::new("."));

                // Пробуем вытащить пути до конфигурации
                let manifest_paths = match manifest.paths {
                    None => {
                        error!("No [paths] table in manifest. Can not find any config.");
                        exit(1);
                    }
                    Some(paths) => paths,
                };

                // Превращаем путь к базовому конфигу в абсолютный, если он относительный
                let base_config_path = if manifest_paths.config.is_relative() {
                    manifest_dir.join(&manifest_paths.config)
                } else {
                    manifest_paths.config.clone()
                };

                // Считываем и парсим базовый конфиг как динамическую таблицу
                let main_toml_str = match std::fs::read_to_string(&base_config_path) {
                    Ok(text) => text,
                    Err(e) => {
                        error!(
                            "Critical failure during loading main config {} : {}",
                            base_config_path.to_string_lossy(),
                            e
                        );
                        exit(1);
                    }
                };

                let mut final_table = match toml::from_str(&main_toml_str) {
                    Ok(table) => table,
                    Err(e) => {
                        error!("Critical failure during parsing main config: {}", e);
                        exit(1);
                    }
                };

                // Накладываем оверлеи поверх
                for overlay_path in manifest_paths.overlays {
                    // Если путь относительный - клеим его к папке манифеста
                    let resolved_overlay_path = if overlay_path.is_relative() {
                        manifest_dir.join(&overlay_path)
                    } else {
                        overlay_path
                    };

                    let overlay_str = match std::fs::read_to_string(&resolved_overlay_path) {
                        Ok(text) => text,
                        Err(err) => {
                            error!(
                                "Failed to read overlay {}: {}",
                                resolved_overlay_path.to_string_lossy(),
                                err
                            );
                            exit(1);
                        }
                    };

                    let overlay_table: toml::Table = match toml::from_str(&overlay_str) {
                        Ok(table) => table,
                        Err(e) => {
                            error!(
                                "Failed to parse overlay TOML {}: {}",
                                resolved_overlay_path.to_string_lossy(),
                                e
                            );
                            exit(1);
                        }
                    };

                    // Мержим секции оверлея в финальную таблицу
                    ConfigLoader::merge_toml_tables(&mut final_table, overlay_table);
                }

                daemon_settings = manifest.daemon_settings;

                final_table
            }
        };

        // Переводим готовую склеенную таблицу в типизированную структуру
        let shadow_root: FullConfigShadow = match final_table.try_into() {
            Ok(root) => root,
            Err(e) => {
                error!(
                    "Critical failure during mapping result toml to shadow struct: {}",
                    e
                );
                exit(1);
            }
        };

        // Достаём критические конфиги
        let (
            Some(settings_shadow),
            Some(screen_reading_shadow),
            Some(led_position_shadow),
            Some(screen_config_shadow),
            Some(frame_connection_config_shadow),
        ) = (
            shadow_root.settings.as_ref(),
            shadow_root.screen_reading_config.as_ref(),
            shadow_root.led_position_config.as_ref(),
            shadow_root.screen_config.as_ref(),
            shadow_root.frame_connection_config.as_ref(),
        )
        else {
            // В случае ошибки прерываем выполнение кода
            panic!(
                "Check sections: \
                \n    [settings]; \
                \n    [screen_reading_config]; \
                \n    [screen_config]; \
                \n    [led_position_config]; \
                \n    [frame_connection_config];"
            );
        };

        // При успехе преобразуем
        let settings: Settings = settings_shadow.into();
        let screen_reading_config: ScreenReadingConfig = screen_reading_shadow.into();
        let led_position_config: LedPositionConfig = led_position_shadow.into();
        let screen_config: ScreenConfig = screen_config_shadow.into();
        let frame_connection_config: FrameConnectionConfig = frame_connection_config_shadow.into();

        // Проверка (screen_config проверим после запуска потока захвата)
        validate_configs(&[&screen_reading_config, &led_position_config], MODULE);

        // Объединяем конфигурацию
        let geometry_config = GeometryConfig {
            led_pos: led_position_config,
            frame_connection: frame_connection_config,
            reading: screen_reading_config,
        };

        // Снова проверяем была ли выбрана проста конфигурация. Если да, то
        // пробуем считать секцию настроек процесса из конфига
        if let Some(daemon_settings_shadow) = shadow_root.daemon_settings.as_ref() {
            daemon_settings = Some(daemon_settings_shadow.into());
        }

        debug!("Config loaded successful");

        Self {
            settings,
            daemon_settings,
            shadow_root,
            geometry_config,
            screen_config,
            cli_flags,
        }
    }

    /// Получение специальных флагов
    pub fn get_flags(&self) -> Flags {
        if let Some(flags) = self.shadow_root.flags.clone() {
            flags.into()
        } else {
            Flags {
                pipewire_conversion: false,
                save_token: false,
            }
        }
    }

    /// Запуск подготовки и основного цикла
    ///
    /// Выполняет действия из флагов, если это возможно на данной ступени
    ///
    /// Данная функция запускает сборку всех компонентов в порядке:
    /// Аналитик -> Фильтр -> Форматтер -> Обходчик -> Поток захвата -> Движок
    pub fn run_stages(self, controller: Arc<CoreController>) {
        //Если есть флаг на очистку сессии, очищаем её
        if self.cli_flags.reset_pipewire_token {
            let session_path = Self::get_session_path();

            if let Err(e) = std::fs::remove_file(&session_path) {
                warn!(
                    "Can not remove session file {}: {}",
                    session_path.to_string_lossy(),
                    e
                );
            }
        }

        // И запускаем обработку по стадиям
        self.stage_1_select_color_analyst(controller);
    }

    /// 1-я ступень - выбор анализатора цвета в фрагменте
    ///
    /// Данная ступень выбирает реализацию трейта [ColorAnalyst]
    ///
    /// **Поддерживается обработка для:**
    /// - [ColorAnalystType::ColorHistogram]
    /// - [ColorAnalystType::ColorAverage]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_1_select_color_analyst(self, controller: Arc<CoreController>) {
        debug!("Running stage 1");
        match self.settings.analytics_type {
            ColorAnalystType::ColorHistogram => {
                let Some(shadow) = self.shadow_root.color_histogram_config.as_ref() else {
                    error!("Check section [color_histogram_config]");
                    exit(1);
                };
                let config: ColorHistogramConfig = shadow.into();

                validate_config(&config, MODULE);

                let analyst = ColorHistogram::new(config);
                self.stage_2_select_filter(controller, analyst);
            }
            ColorAnalystType::ColorAverage => {
                let analyst = ColorAverage::new();
                self.stage_2_select_filter(controller, analyst);
            }
        }
    }

    /// 2-я ступень - выбор фильтра для результирующего цвета
    ///
    /// Данная ступень реализует выбор трейта [ColorFilter]
    ///
    /// **Поддерживается обработка для:**
    /// - [ColorFilterType::EmaFilter]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_2_select_filter<Analyst>(self, controller: Arc<CoreController>, analyst: Analyst)
    where
        Analyst: ColorAnalyst,
    {
        debug!("Running stage 2");

        // Если массив пуст
        if self.settings.filter_chain.len() == 0 {
            self.stage_3_run_capture_thread(controller, analyst, NoFilter::new());
            return;
        }

        // Иначе генерируем цепь
        let mut filter_chain = FilterChain::new();

        for filter_type in self.settings.filter_chain.iter() {
            let instance = match filter_type {
                ColorFilterType::NoFilter => {
                    warn!("NoFilter missed in the chain.");
                    continue;
                }

                ColorFilterType::EmaFilter => {
                    let Some(shadow) = self.shadow_root.ema_filter_config.as_ref() else {
                        error!("Check section [ema_filter_config]");
                        exit(2);
                    };
                    let shadow_config: EmaFilterConfig = shadow.into();
                    let config = EmaFilterConfig {
                        alpha: shadow_config.alpha,
                        amount: self.geometry_config.calculate_leds_amount(),
                    };

                    validate_config(&config, MODULE);

                    let filter = EmaFilter::new(config);
                    FilterInstance::Ema(filter)
                }

                ColorFilterType::GammaFilter => {
                    let Some(shadow) = self.shadow_root.gamma_filter_config.as_ref() else {
                        error!("Check section [gamma_filter_config]");
                        exit(2);
                    };
                    let config: GammaFilterConfig = shadow.into();

                    validate_config(&config, MODULE);

                    let filter = GammaFilter::new(config);
                    FilterInstance::Gamma(filter)
                }
            };

            filter_chain.add_filter(instance);
        }

        self.stage_3_run_capture_thread(controller, analyst, filter_chain);
    }

    /// 3-я ступень - запуск потока захвата
    ///
    /// Данная ступень безопасно запускает [CaptureThread]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_3_run_capture_thread<Analyst, Filter>(
        self,
        controller: Arc<CoreController>,
        analyst: Analyst,
        filter: Filter,
    ) where
        Analyst: ColorAnalyst,
        Filter: ColorFilter,
    {
        debug!("Running stage 3");

        let mut capture_config = CaptureConfig::new();

        // Берем указатель
        let token = if let Ok(encrypted_bytes) = std::fs::read(Self::get_session_path()) {
            let decrypted_bytes = xor_crypt(&encrypted_bytes);

            if let Ok(token_str) = String::from_utf8(decrypted_bytes) {
                token_str
            } else {
                warn!(
                    "Can not get token from {} file.\nIt could be corrupted",
                    Self::get_session_path().to_string_lossy()
                );
                String::new()
            }
        } else {
            info!(
                "Can not open {} file.",
                Self::get_session_path().to_string_lossy()
            );
            String::new()
        };

        let token_ptr = if !token.is_empty() {
            match CString::new(token) {
                Ok(c) => c.into_raw(),
                Err(e) => {
                    error!("Failed to create CString {}", e);
                    exit(3)
                }
            }
        } else {
            std::ptr::null_mut()
        };

        debug!("token_ptr is null: {}", token_ptr.is_null());
        debug!("apply_conversion: {}", self.get_flags().pipewire_conversion);

        let mut capture_thread = match CaptureThread::new(
            &mut capture_config,
            controller.clone(),
            InitializingData {
                apply_conversion: self.get_flags().pipewire_conversion,
                token: token_ptr,
            },
        ) {
            Ok(thread) => thread,
            Err(e) => {
                error!("{}", e);
                exit(3); // Выходим с кодом ошибки
            }
        };

        // Ждем запуска
        while !controller.keep_running() {
            // Спим 10мс, чтобы не грузить CPU
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Сразу после вызова (так как Си уже сделал strdup внутри new),
        // мы возвращаем указатель в Rust, чтобы он его удалил и не было утечки:
        if !token_ptr.is_null() {
            unsafe {
                let _ = CString::from_raw(token_ptr);
            }
        }

        self.stage_4_select_formatter(capture_config, &mut capture_thread, analyst, filter);
    }

    /// 4-я ступень - выбор преобразователя пикселей
    ///
    /// Данная ступень выбирает реализацию трейта [PixelFormatter] в зависимости
    /// от формата, заданного в конфиге, который будет согласован с потоком захвата
    ///
    /// **Поддерживается обработка для:**
    /// - [SpaVideoFormat::ABGR]
    /// - [SpaVideoFormat::ARGB]
    /// - [SpaVideoFormat::xBGR]
    /// - [SpaVideoFormat::xRGB]
    /// - [SpaVideoFormat::BGRA]
    /// - [SpaVideoFormat::RGBA]
    /// - [SpaVideoFormat::BGRx]
    /// - [SpaVideoFormat::RGBx]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_4_select_formatter<Analyst, Filter>(
        self,
        capture_config: CaptureConfig,
        capture_thread: &mut CaptureThread,
        analyst: Analyst,
        filter: Filter,
    ) where
        Analyst: ColorAnalyst,
        Filter: ColorFilter,
    {
        debug!("Running stage 4");

        let format = match SpaVideoFormat::try_from(capture_config.format()) {
            Ok(f) => f,
            Err(f) => {
                error!("Strange video format id: {}", f);
                capture_thread.stop();
                exit(4);
            }
        };

        match format {
            SpaVideoFormat::RGBx => {
                self.stage_5_select_chunk_processor::<RGBx, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }
            SpaVideoFormat::BGRx => {
                self.stage_5_select_chunk_processor::<BGRx, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }
            SpaVideoFormat::xRGB => {
                self.stage_5_select_chunk_processor::<xRGB, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }
            SpaVideoFormat::xBGR => {
                self.stage_5_select_chunk_processor::<xBGR, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }
            SpaVideoFormat::RGBA => {
                self.stage_5_select_chunk_processor::<RGBA, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }
            SpaVideoFormat::BGRA => {
                self.stage_5_select_chunk_processor::<BGRA, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }
            SpaVideoFormat::ARGB => {
                self.stage_5_select_chunk_processor::<ARGB, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }
            SpaVideoFormat::ABGR => {
                self.stage_5_select_chunk_processor::<ABGR, _, _>(
                    capture_config,
                    capture_thread,
                    analyst,
                    filter,
                );
            }

            format => {
                error!(
                    "Format {} not supported. Try setting the flag `pipewire_conversion = true` in the section `[settings.flags]`",
                    format
                );
                capture_thread.stop();
                exit(4);
            }
        };
    }

    /// 5-я ступень - выбор обработчика фрагментов
    ///
    /// Данная ступень выбирает реализацию трейта [ChunkProcessor], также
    /// производит размер кадра в байтах для потока захвата с помощью информации
    /// из [PixelFormatter] (а именно [PixelFormatter::SIZE] - размер пикселя в
    /// байтах)
    ///
    /// **Поддерживается обработка для:**
    /// - [ChunkProcessorType::Checkerboard]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_5_select_chunk_processor<Formatter, Analyst, Filter>(
        mut self,
        capture_config: CaptureConfig,
        capture_thread: &mut CaptureThread,
        analyst: Analyst,
        filter: Filter,
    ) where
        Formatter: PixelFormatter,
        Analyst: ColorAnalyst,
        Filter: ColorFilter,
    {
        debug!("Running stage 5");

        // Подтягиваем конфигурацию экрана из потока захвата
        self.screen_config
            .load_px(capture_config.width(), capture_config.height());

        // Проверяем, как и обещали
        match self.screen_config.validate() {
            Ok(warnings) => warn_validate(warnings, MODULE),
            Err(error) => {
                error!("{}", error);
                capture_thread.stop();
                exit(5)
            }
        }

        let flags = self.get_flags();
        if flags.save_token {
            let token = match capture_thread.get_token() {
                None => Vec::new(),
                Some(s) => s.into_bytes(),
            };

            let encrypted_bytes = xor_crypt(&token);

            if let Err(e) = std::fs::write(Self::get_session_path(), encrypted_bytes) {
                warn!(
                    "Can not write {}: {}",
                    Self::get_session_path().to_string_lossy(),
                    e
                );
            }
        }

        // Выбираем тип обработчика фрагмента из конфига
        match self.settings.chunk_processor_type {
            ChunkProcessorType::Checkerboard => {
                let Some(shadow) = self.shadow_root.checkerboard_config.as_ref() else {
                    error!("Check section [checkerboard_config]");
                    capture_thread.stop();
                    exit(5);
                };

                let mut alg_config: CheckerboardConfig = shadow.into();
                alg_config.config = self
                    .geometry_config
                    .calculate_chunk_config(self.screen_config);

                match alg_config.validate() {
                    Ok(warnings) => warn_validate(warnings, MODULE),
                    Err(error) => {
                        error!("{}", error);
                        capture_thread.stop();
                        exit(5);
                    }
                }

                let processor = CheckerboardScanner::new(alg_config, self.screen_config);

                self.stage_6_select_hardware_driver::<Formatter, _, _, _>(
                    capture_thread,
                    processor,
                    analyst,
                    filter,
                );
            }
            ChunkProcessorType::DynamicCheckerboard => {
                let Some(shadow) = self.shadow_root.dynamic_checkerboard_config.as_ref() else {
                    error!("Check section [dynamic_checkerboard_config]");
                    capture_thread.stop();
                    exit(5);
                };

                let mut alg_config: DynamicCheckerboardConfig = shadow.into();
                alg_config.config = self
                    .geometry_config
                    .calculate_chunk_config(self.screen_config);

                // match alg_config.validate() {
                //     Ok(warnings) => warn_validate(warnings, MODULE),
                //     Err(error) => {
                //         error!("{}", error);
                //         capture_thread.stop();
                //         exit(5);
                //     }
                // }

                let processor = DynamicCheckerboardScanner::new(alg_config, self.screen_config);

                self.stage_6_select_hardware_driver::<Formatter, _, _, _>(
                    capture_thread,
                    processor,
                    analyst,
                    filter,
                );
            }
        }
    }

    /// 6-я ступень - сборка движка и выбор вывода на устройство
    ///
    /// Данная ступень выбирает реализацию трейта [hardware_output::HardwareOutput]
    ///
    /// **Поддерживается реализация для:**
    /// - [HardwareOutputType::DebugDriver]
    /// - [HardwareOutputType::SerialDriver]
    ///
    /// После обработки запускается основной цикл, содержащийся в `main`
    fn stage_6_select_hardware_driver<Formatter, Processor, Analyst, Filter>(
        self,
        capture_thread: &mut CaptureThread,
        processor: Processor,
        analyst: Analyst,
        filter: Filter,
    ) where
        Formatter: PixelFormatter,
        Processor: ChunkProcessor<Formatter, Analyst>,
        Analyst: ColorAnalyst,
        Filter: ColorFilter,
    {
        debug!("Running stage 6");

        let led_amount = self.geometry_config.calculate_leds_amount();
        let hardware_output_type = self.settings.hardware_output_type;

        // Извлекаем нужные параметры из geometry_config до того, как он "уедет" в color_engine
        let hor_amount = self.geometry_config.led_pos.horizontal_led_amount;
        let ver_amount = self.geometry_config.led_pos.vertical_led_amount;

        // Мы передаём владение geometry_config и screen_config в движок
        let color_engine = ColorEngine::<Formatter, _, _, _>::new(
            processor,
            analyst,
            filter,
            self.geometry_config, // Перенос владения
            self.screen_config,   // Перенос владения
        );

        match hardware_output_type {
            HardwareOutputType::DebugDriver => {
                let hardware_output = DebugDriver::new(hor_amount, ver_amount);

                // Оставшиеся лишние данные будут уничтожены при выходе из этой функции
                run_ambient_loop(color_engine, hardware_output, led_amount, capture_thread);
            }

            HardwareOutputType::SerialDriver => {
                let Some(shadow) = self.shadow_root.serial_driver_config else {
                    error!("Check section [serial_driver_config]");
                    capture_thread.stop();
                    exit(6);
                };
                let serial_driver_config: SerialDriverConfig = shadow.into();

                match serial_driver_config.validate() {
                    Ok(warnings) => warn_validate(warnings, MODULE),
                    Err(error) => {
                        error!("{}", error);
                        capture_thread.stop();
                        exit(6);
                    }
                }

                let hardware_output = match SerialDriver::new(serial_driver_config) {
                    Ok(h) => h,
                    Err(error) => {
                        error!("{}", error);
                        capture_thread.stop();
                        exit(6);
                    }
                };

                // В этот момент всё лишнее уничтожается
                run_ambient_loop(color_engine, hardware_output, led_amount, capture_thread);
            }
        }
    }
}
