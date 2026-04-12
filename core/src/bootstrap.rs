//! Модуль загрузки данных из конфига и подготовки структур для запуска цикла
//!
//! Данный модуль вытягивает данные из toml-файла и запускает сборку данных по ступеням:
//! 1) выбор обработчика фрагментов
//! 2) выбор анализатора цвета в фрагменте
//! 3) выбор фильтра для результирующего цвета
//! 4) выбор способа вывода на устройство
//!
//! После подготовки всех ступеней запускается основной цикл программы,
//! содержащийся в main

use crate::config::Settings;
use crate::run_ambient_loop;
use algorithms::{
    analytics::{registry::*, types::*, ColorAnalyst},
    filters::{registry::*, types::*, ColorFilter},
    processing::{configs::*, registry::*, types::*, ChunkProcessor},
    units::*,
};
use config_gen::{__private::*, *};
use hardware_output::{
    debug::types::DebugDriver,
    registry::*,
    serial::{config::SerialDriverConfig, types::SerialDriver},
};
use threads::screen_capture::screen_capture::CaptureThread;

include_shadow_all!(
    "./algorithms/src/units.rs",
    "./algorithms/src/processing/configs.rs",
    "./algorithms/src/processing/registry.rs",
    "./algorithms/src/analytics/registry.rs",
    "./algorithms/src/filters/registry.rs",
    "./hardware_output/src/registry.rs",
    "./core/src/config.rs",
    "./hardware_output/src/serial/config.rs"
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
    /// 3) TODO: проверить эти настройки
    ///
    /// Далее она преобразует их в готовые для работы данные, которые необходимы
    /// для запуска дальнейшей инициализации
    pub fn load() -> Self {
        // Считываем данные из файла
        let toml_str = std::fs::read_to_string("cfg.toml").expect("Не удалось прочитать cfg.toml");

        // Проводим десериализацию
        let shadow_root: FullConfigShadow =
            toml::from_str(&toml_str).expect("Ошибка парсинга TOML");

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
            panic!("Проверьте секции: 
                    \n    [settings]; 
                    \n    [screen_reading_config]; 
                    \n    [screen_config];
                    \n    [led_position_config]");
        };

        // При успехе преобразуем
        let settings: Settings = settings_shadow.into();
        let screen_reading_config: ScreenReadingConfig = screen_reading_shadow.into();
        let led_position_config: LedPositionConfig = led_position_shadow.into();
        let screen_config: ScreenConfig = screen_config_shadow.into();

        // TODO! Проверка данных из критических конфигов!!!
        // TOFO! Проверка наличия секций для выбранных алгоритмов!!

        // Объединяем конфигурацию
        let geometry_config = GeometryConfig {
            led_pos: led_position_config,
            reading: screen_reading_config,
        };

        Self {
            settings,
            shadow_root,
            geometry_config,
            screen_config,
        }
    }

    /// Запуск подготовки и основного цикла
    ///
    /// Данная функция подтягивает необходимую конфигурацию в настройки конфигурации
    /// экрана и запускает следующую ступень
    pub fn run_stages(
        mut self,
        screen_width: u32,
        screen_height: u32,
        capture_thread: &CaptureThread,
    ) {
        // Подтягиваем конфигурацию экрана из потока захвата
        self.screen_config.load_px(screen_width, screen_height);

        // И запускаем обработку по стадиям
        self.stage_1_select_chunk_processor(capture_thread);
    }

    /// 1-я ступень - выбор обработчика фрагментов
    ///
    /// Данная ступень выбирает реализацию трейта [ChunkProcessor]
    ///
    /// **Поддерживается обработка для:**
    /// - [ChunkProcessorType::Checkerboard]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_1_select_chunk_processor(self, capture_thread: &CaptureThread) {
        // Выбираем тип обработчика фрагмента из конфига
        match self.settings.chunk_processor_type {
            ChunkProcessorType::Checkerboard => {
                let Some(shadow) = self.shadow_root.checkerboard_config.as_ref() else {
                    println!("Проверьте секцию [checkerboard_config]");
                    return;
                };

                let mut alg_config: CheckerboardConfig = shadow.into();
                alg_config.config = self
                    .geometry_config
                    .calculate_chunk_config(self.screen_config);

                let processor = CheckerboardScanner::new(alg_config, self.screen_config);

                // TODO! Проверка конфига!!

                self.stage_2_select_color_analyst(capture_thread, processor);
            }
        }
    }

    /// 2-я ступень - выбор анализатора цвета в фрагменте
    ///
    /// Данная ступень выбирает реализацию трейта [ColorAnalyst]
    ///
    /// **Поддерживается обработка для:**
    /// - [ColorAnalystType::ColorHistogram]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_2_select_color_analyst<Processor>(
        self,
        capture_thread: &CaptureThread,
        processor: Processor,
    ) where
        Processor: ChunkProcessor,
    {
        match self.settings.analytics_type {
            ColorAnalystType::ColorHistogram => {
                let analyst = ColorHistogram::new();
                self.stage_3_select_filter(capture_thread, processor, analyst);
            }
        }
    }

    /// 3-я ступень - выбор фильтра для результирующего цвета
    ///
    /// Данная ступень реализует выбор трейта [ColorFilter]
    ///
    /// **Поддерживается обработка для:**
    /// - [ColorFilterType::EmaFilter]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_3_select_filter<Processor, Analyst>(
        self,
        capture_thread: &CaptureThread,
        processor: Processor,
        analyst: Analyst,
    ) where
        Processor: ChunkProcessor,
        Analyst: ColorAnalyst,
    {
        match self.settings.filter_type {
            ColorFilterType::EmaFilter => {
                let filter = EmaFilter::new(self.geometry_config.calculate_leds_amount());
                self.stage_4_select_hardware_driver(capture_thread, processor, analyst, filter);
            }
        }
    }

    /// 4-я ступень - выбор вывода на устройство
    ///
    /// Данная ступень выбирает реализацию трейта [hardware_output::HardwareOutput]
    ///
    /// **Поддерживается реализация для:**
    /// - [HardwareOutputType::DebugDriver]
    /// - [HardwareOutputType::SerialDriver]
    ///
    /// После обработки запускается основной цикл, содержащийся в `main`
    fn stage_4_select_hardware_driver<Processor, Analyst, Filter>(
        self,
        capture_thread: &CaptureThread,
        processor: Processor,
        analyst: Analyst,
        filter: Filter,
    ) where
        Processor: ChunkProcessor,
        Analyst: ColorAnalyst,
        Filter: ColorFilter,
    {
        let led_amount = self.geometry_config.calculate_leds_amount();
        let hardware_output_type = self.settings.hardware_output_type;

        // Извлекаем нужные параметры из geometry_config до того, как он "уедет" в color_engine
        let hor_amount = self.geometry_config.led_pos.horizontal_led_amount;
        let ver_amount = self.geometry_config.led_pos.vertical_led_amount;

        // Мы передаём владение geometry_config и screen_config в движок
        let color_engine = ColorEngine::new(
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
                    println!("Проверьте секцию [serial_driver_config]");
                    return;
                };
                let serial_driver_config: SerialDriverConfig = shadow.into();
                let hardware_output = SerialDriver::new(serial_driver_config);

                // В этот момент всё лишнее уничтожается
                run_ambient_loop(color_engine, hardware_output, led_amount, capture_thread);
            }
        }
    }
}
