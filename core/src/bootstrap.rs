//! Модуль загрузки данных из конфига и подготовки структур для запуска цикла
//!
//! Данный модуль вытягивает данные из toml-файла и запускает сборку данных по ступеням:
//! 1) выбор анализатора цвета в фрагменте
//! 2) выбор фильтра для результирующего цвета
//! 3) выбор преобразователя пикселей
//! 4) выбор обработчика фрагментов + запуск потока захвата
//! 5) сборка движка и запуск основного цикла программы,
//!    содержащегося в main
//!
//! После подготовки всех ступеней запускается основной цикл программы,
//! содержащийся в main

const MODULE: &str = "ConfigLoader";

use crate::config::*;
use crate::run_ambient_loop;
use algorithms::{
    analytics::{configs::*, registry::*, types::*, ColorAnalyst},
    filters::{configs::*, registry::*, types::*, ColorFilter},
    pixel_formatter::{types::*, PixelFormatter},
    processing::{
        configs::*,
        processors::{configs::*, registry::*, types::*, ChunkProcessor, Orientation},
        types::*,
    },
};
use common::{configs::*, core::controller::CoreController, units::*};
use config_gen::{__private::*, *};
use ffi::bindings::{CaptureConfig, InitializingData, SpaVideoFormat};
use hardware_output::{
    debug::types::DebugDriver,
    registry::*,
    serial::{config::SerialDriverConfig, types::SerialDriver},
};
use logger::*;
use std::{ffi::CString, process::exit, sync::Arc};
use threads::screen_capture::screen_capture::CaptureThread;

include_shadow_all!(
    "./common/src/units/units.rs"
    "./algorithms/src/processing/configs/configs.rs",
    "./algorithms/src/processing/processors/registry.rs",
    "./algorithms/src/processing/processors/configs/configs.rs",
    "./algorithms/src/analytics/registry.rs",
    "./algorithms/src/analytics/configs/configs.rs",
    "./algorithms/src/filters/registry.rs",
    "./algorithms/src/filters/configs/configs.rs",
    "./hardware_output/src/registry.rs",
    "./core/src/config.rs",
    "./hardware_output/src/serial/config/config.rs"
);

pub struct ConfigLoader {
    settings: Settings,
    shadow_root: FullConfigShadow, // Храним временно для инициализации алгоритмов
    geometry_config: GeometryConfig,
    screen_config: ScreenConfig,
}

impl ConfigLoader {
    /// Инициализация конфигурации
    ///
    /// Данная функция пытается:
    /// 1) прочитать данные из файла
    /// 2) извлечь критически важные настройки
    /// 3) проверить эти настройки
    ///
    /// Далее она преобразует их в готовые для работы данные, которые необходимы
    /// для запуска дальнейшей инициализации
    pub fn load() -> Self {
        // Считываем данные из файла
        let toml_str = match std::fs::read_to_string("cfg.toml") {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Critical failure during loading: {}", e);
                exit(1);
            }
        };

        // Проводим десериализацию
        let shadow_root: FullConfigShadow = match toml::from_str(&toml_str) {
            Ok(root) => root,
            Err(e) => {
                error!("Critical failure during reading toml: {}", e);
                exit(1);
            }
        };

        // Достаём критические конфиги
        let (
            Some(settings_shadow),
            Some(screen_reading_shadow),
            Some(led_position_shadow),
            Some(screen_config_shadow),
        ) = (
            shadow_root.settings.as_ref(),
            shadow_root.screen_reading_config.as_ref(),
            shadow_root.led_position_config.as_ref(),
            shadow_root.screen_config.as_ref(),
        )
        else {
            // В случае ошибки прерываем выполнение кода
            panic!(
                "Check sections: 
                    \n    [settings]; 
                    \n    [screen_reading_config]; 
                    \n    [screen_config];
                    \n    [led_position_config]"
            );
        };

        // При успехе преобразуем
        let settings: Settings = settings_shadow.into();
        let screen_reading_config: ScreenReadingConfig = screen_reading_shadow.into();
        let led_position_config: LedPositionConfig = led_position_shadow.into();
        let screen_config: ScreenConfig = screen_config_shadow.into();

        // Проверка (screen_config проверим после запуска потока захвата)
        validate_configs(&[&screen_reading_config, &led_position_config], MODULE);

        // Объединяем конфигурацию
        let geometry_config = GeometryConfig {
            led_pos: led_position_config,
            reading: screen_reading_config,
        };

        debug!("Config loaded successful");

        Self {
            settings,
            shadow_root,
            geometry_config,
            screen_config,
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
    /// Данная функция запускает сборку всех компонентов в порядке:
    /// Аналитик -> Фильтр -> Форматтер -> Обходчик -> Поток захвата -> Движок
    pub fn run_stages(self, controller: Arc<CoreController>) {
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
        let token_ptr = if !self.settings.session_token.is_empty() {
            match CString::new(self.settings.session_token.clone()) {
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
        if let Some(ref mut settings) = self.shadow_root.settings {
            if flags.save_token {
                let token = match capture_thread.get_token() {
                    None => "",
                    Some(s) => &s.to_string(),
                };

                settings.session_token = token.to_string();

                // Записываем обновлённый конфиг на диск
                match toml::to_string_pretty(&self.shadow_root) {
                    Ok(toml_str) => {
                        if let Err(e) = std::fs::write("cfg.toml", toml_str) {
                            error!("Failed to save token: {}", e);
                        }
                    }
                    Err(e) => error!("Failed to serialize config: {}", e),
                }
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
        Processor: ChunkProcessor<Formatter>,
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
