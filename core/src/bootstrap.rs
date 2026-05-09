//! Модуль загрузки данных из конфига и подготовки структур для запуска цикла
//!
//! Данный модуль вытягивает данные из toml-файла и запускает сборку данных по ступеням:
//! 1) выбор преобразователя пикселей
//! 2) выбор обработчика фрагментов
//! 3) выбор анализатора цвета в фрагменте
//! 4) выбор фильтра для результирующего цвета
//! 5) выбор способа вывода на устройство
//!
//! После подготовки всех ступеней запускается основной цикл программы,
//! содержащийся в main

use crate::config::Flags;
use crate::config::Settings;
use crate::run_ambient_loop;
use algorithms::analytics::configs::ColorHistogramConfig;
use algorithms::filters::configs::EmaFilterConfig;
use algorithms::filters::configs::GammaFilterConfig;
use algorithms::{
    analytics::{registry::*, types::*, ColorAnalyst},
    filters::{registry::*, types::*, ColorFilter},
    pixel_formatter::{
        types::{xBGR, xRGB, BGRx, RGBx, ABGR, ARGB, BGRA, RGBA},
        PixelFormatter,
    },
    processing::{
        configs::*,
        processors::{
            configs::{CheckerboardConfig, ChunkConfig, ChunkTask},
            registry::*,
            types::CheckerboardScanner,
            ChunkProcessor,
        },
        types::*,
    },
    units::*,
};
use config_gen::{__private::*, *};
use ffi::bindings::SpaVideoFormat;
use hardware_output::{
    debug::types::DebugDriver,
    registry::*,
    serial::{config::SerialDriverConfig, types::SerialDriver},
};
use threads::screen_capture::screen_capture::CaptureThread;

include_shadow_all!(
    "./algorithms/src/units.rs",
    "./algorithms/src/processing/configs.rs",
    "./algorithms/src/processing/processors/registry.rs",
    "./algorithms/src/processing/processors/configs.rs",
    "./algorithms/src/analytics/registry.rs",
    "./algorithms/src/analytics/configs.rs",
    "./algorithms/src/filters/registry.rs",
    "./algorithms/src/filters/configs.rs",
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
        let toml_str = std::fs::read_to_string("cfg.toml").expect("Failed to read cfg.toml");

        // Проводим десериализацию
        let shadow_root: FullConfigShadow = toml::from_str(&toml_str).expect("Parsing error TOML");

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

        // TODO! Проверка данных из критических конфигов!!!
        // TODO! Проверка наличия секций для выбранных алгоритмов!!

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

    /// Получение специальных флагов
    pub fn get_flags(&self) -> Flags {
        Flags {
            pipewire_conversion: self.settings.flags.pipewire_conversion,
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
        video_format: u32,
        capture_thread: &CaptureThread,
    ) {
        // Подтягиваем конфигурацию экрана из потока захвата
        self.screen_config.load_px(screen_width, screen_height);

        // И запускаем обработку по стадиям
        self.stage_1_select_formatter(video_format, capture_thread);
    }

    /// 1-я ступень - выбор преобразователя пикселей
    ///
    /// Данная ступень выбирает реализацию трейта [PixelFormatter] в зависимости
    /// от [SpaVideoFormat]
    ///
    /// **Поддерживается обработка для:**
    /// - [SpaVideoFormat::BGRA]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_1_select_formatter(self, video_format: u32, capture_thread: &CaptureThread) {
        let format = SpaVideoFormat::try_from(video_format)
            .expect(format!("Strange video format id: {}", video_format).as_str());

        match format {
            SpaVideoFormat::RGBx => {
                self.stage_2_select_chunk_processor::<RGBx>(capture_thread);
            }
            SpaVideoFormat::BGRx => {
                self.stage_2_select_chunk_processor::<BGRx>(capture_thread);
            }
            SpaVideoFormat::xRGB => {
                self.stage_2_select_chunk_processor::<xRGB>(capture_thread);
            }
            SpaVideoFormat::xBGR => {
                self.stage_2_select_chunk_processor::<xBGR>(capture_thread);
            }
            SpaVideoFormat::RGBA => {
                self.stage_2_select_chunk_processor::<RGBA>(capture_thread);
            }
            SpaVideoFormat::BGRA => {
                self.stage_2_select_chunk_processor::<BGRA>(capture_thread);
            }
            SpaVideoFormat::ARGB => {
                self.stage_2_select_chunk_processor::<ARGB>(capture_thread);
            }
            SpaVideoFormat::ABGR => {
                self.stage_2_select_chunk_processor::<ABGR>(capture_thread);
            }

            format => {
                println!(
                    "[ERROR] ConfigLoader: Format {} not supported. Try setting the flag `pipewire_conversion = true` in the section `[settings.flags]`",
                    format
                );
                return;
            }
        };
    }

    /// 2-я ступень - выбор обработчика фрагментов
    ///
    /// Данная ступень выбирает реализацию трейта [ChunkProcessor]
    ///
    /// **Поддерживается обработка для:**
    /// - [ChunkProcessorType::Checkerboard]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_2_select_chunk_processor<Formatter>(self, capture_thread: &CaptureThread)
    where
        Formatter: PixelFormatter,
    {
        // Выбираем тип обработчика фрагмента из конфига
        match self.settings.chunk_processor_type {
            ChunkProcessorType::Checkerboard => {
                let Some(shadow) = self.shadow_root.checkerboard_config.as_ref() else {
                    println!("[ERROR] ConfigLoader: Check section [checkerboard_config]");
                    return;
                };

                let mut alg_config: CheckerboardConfig = shadow.into();
                alg_config.config = self
                    .geometry_config
                    .calculate_chunk_config(self.screen_config);

                let processor = CheckerboardScanner::new(alg_config, self.screen_config);

                // TODO! Проверка конфига!!

                self.stage_3_select_color_analyst::<Formatter, _>(capture_thread, processor);
            }
        }
    }

    /// 3-я ступень - выбор анализатора цвета в фрагменте
    ///
    /// Данная ступень выбирает реализацию трейта [ColorAnalyst]
    ///
    /// **Поддерживается обработка для:**
    /// - [ColorAnalystType::ColorHistogram]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_3_select_color_analyst<Formatter, Processor>(
        self,
        capture_thread: &CaptureThread,
        processor: Processor,
    ) where
        Formatter: PixelFormatter,
        Processor: ChunkProcessor<Formatter>,
    {
        match self.settings.analytics_type {
            ColorAnalystType::ColorHistogram => {
                let Some(shadow) = self.shadow_root.color_histogram_config.as_ref() else {
                    println!("[ERROR] ConfigLoader: Check section [color_histogram_config]");
                    return;
                };

                let analyst = ColorHistogram::new(shadow.into());
                self.stage_4_select_filter::<Formatter, _, _>(capture_thread, processor, analyst);
            }
        }
    }

    /// 4-я ступень - выбор фильтра для результирующего цвета
    ///
    /// Данная ступень реализует выбор трейта [ColorFilter]
    ///
    /// **Поддерживается обработка для:**
    /// - [ColorFilterType::EmaFilter]
    ///
    /// После подготовки запускается следующая ступень
    fn stage_4_select_filter<Formatter, Processor, Analyst>(
        self,
        capture_thread: &CaptureThread,
        processor: Processor,
        analyst: Analyst,
    ) where
        Formatter: PixelFormatter,
        Processor: ChunkProcessor<Formatter>,
        Analyst: ColorAnalyst,
    {
        // Если массив пуст
        if self.settings.filter_chain.len() == 0 {
            self.stage_5_select_hardware_driver::<Formatter, _, _, _>(
                capture_thread,
                processor,
                analyst,
                NoFilter::new(),
            );
            return;
        }

        // Иначе генерируем цепь
        let mut filter_chain = FilterChain::new();

        for filter_type in self.settings.filter_chain.iter() {
            let instance = match filter_type {
                ColorFilterType::NoFilter => {
                    println!("[WARN] ConfigLoader: NoFilter missed in the chain.");
                    continue;
                }

                ColorFilterType::EmaFilter => {
                    let Some(shadow) = self.shadow_root.ema_filter_config.as_ref() else {
                        println!("[ERROR] ConfigLoader: Check section [ema_filter_config]");
                        return;
                    };
                    let shadow_config: EmaFilterConfig = shadow.into();
                    let config = EmaFilterConfig {
                        alpha: shadow_config.alpha,
                        amount: self.geometry_config.calculate_leds_amount(),
                    };
                    let filter = EmaFilter::new(config);
                    FilterInstance::Ema(filter)
                }

                ColorFilterType::GammaFilter => {
                    let Some(shadow) = self.shadow_root.gamma_filter_config.as_ref() else {
                        println!("[Error] ConfigLoader: Check section [gamma_filter_config]");
                        return;
                    };
                    let filter = GammaFilter::new(shadow.into());
                    FilterInstance::Gamma(filter)
                }
            };

            filter_chain.add_filter(instance);
        }
        self.stage_5_select_hardware_driver::<Formatter, _, _, _>(
            capture_thread,
            processor,
            analyst,
            filter_chain,
        );
    }

    /// 5-я ступень - выбор вывода на устройство
    ///
    /// Данная ступень выбирает реализацию трейта [hardware_output::HardwareOutput]
    ///
    /// **Поддерживается реализация для:**
    /// - [HardwareOutputType::DebugDriver]
    /// - [HardwareOutputType::SerialDriver]
    ///
    /// После обработки запускается основной цикл, содержащийся в `main`
    fn stage_5_select_hardware_driver<Formatter, Processor, Analyst, Filter>(
        self,
        capture_thread: &CaptureThread,
        processor: Processor,
        analyst: Analyst,
        filter: Filter,
    ) where
        Formatter: PixelFormatter,
        Processor: ChunkProcessor<Formatter>,
        Analyst: ColorAnalyst,
        Filter: ColorFilter,
    {
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
                    println!("[ERROR] ConfigLoader: Check section [serial_driver_config]");
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
