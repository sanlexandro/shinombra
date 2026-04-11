use algorithms::{
    analytics::{registry::*, types::*, ColorAnalyst},
    filters::{registry::*, types::*, ColorFilter},
    processing::{configs::*, registry::*, types::*, ChunkProcessor},
    units::*,
};
use ambient_core::config::Settings;
use config_gen::{__private::*, *};
use ffi::bindings::CaptureConfig;
use hardware_output::{
    debug::types::DebugDriver,
    registry::*,
    serial::{config::SerialDriverConfig, types::SerialDriver},
    HardwareOutput,
};
use std::{sync::atomic::{AtomicBool, Ordering}};
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

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

// ф-я остановки основного потока
fn ctrlc_func() {
    println!("Got signal");
    KEEP_RUNNING.store(false, Ordering::Relaxed);
}

fn main() {
    println!("Ambient Lighting Backend");

    // Обработчик прерывания
    ctrlc::set_handler(ctrlc_func).expect("Some errors!");

    // Считываем данные из файл
    let toml_str = std::fs::read_to_string("cfg.toml").expect("Не удалось прочитать cfg.toml");

    // Проводим десериализацию
    let shadow_root: FullConfigShadow = toml::from_str(&toml_str).expect("Ошибка парсинга TOML");

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
        println!("Проверьте секции [settings], [screen_reading_config], [screen_config] и [led_position_config]");
        return;
    };

    // При успехе преобразуем
    let settings: Settings = settings_shadow.into();
    let screen_reading_config: ScreenReadingConfig = screen_reading_shadow.into();
    let led_position_config: LedPositionConfig = led_position_shadow.into();
    let mut screen_config: ScreenConfig = screen_config_shadow.into();

    // TODO! Проверка данных из критических конфигов!!!
    // TOFO! Проверка наличия секций для выбранных алгоритмов!!

    // Объединяем конфигурацию
    let geometry_config = GeometryConfig {
        led_pos: led_position_config,
        reading: screen_reading_config,
    };

    // println!("settings = {:?}\nscreen_reading_config = {:?}\nscreen_config = {:?}\nled_position_config = {:?}", settings, screen_reading_config, screen_config, led_position_config);

    // Запускаем поток захвата
    let mut capture_config = CaptureConfig::new();

    let mut capture_thread = CaptureThread::new();
    capture_thread.start(&mut capture_config);

    // Ждем, пока флаг станет TRUE
    while !capture_config.is_ready.load(Ordering::Relaxed) {
        // Спим 10мс, чтобы не грузить CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Подтягиваем конфигурацию экрана из потока захвата
    screen_config.load_px(capture_config.screen_width, capture_config.screen_height);

    stage_1_select_chunk_processor(
        settings,
        shadow_root,
        geometry_config,
        screen_config,
        &capture_thread,
    );
    println!("Shutting down...");

    capture_thread.stop();
}

/// 1-я ступень - выбор обработчика фрагментов
fn stage_1_select_chunk_processor(
    settings: Settings,
    shadow_root: FullConfigShadow,
    geometry_config: GeometryConfig,
    screen_config: ScreenConfig,
    capture_thread: &CaptureThread,
) {
    // Выбираем тип обработчика фрагмента из конфига
    match settings.chunk_processor_type {
        ChunkProcessorType::Checkerboard => {
            let Some(shadow) = shadow_root.checkerboard_config.as_ref() else {
                println!("Проверьте секцию [checkerboard_config]");
                return;
            };

            let mut alg_config: CheckerboardConfig = shadow.into();

            alg_config.config = geometry_config.calculate_chunk_config(screen_config);

            let processor = CheckerboardScanner::new(alg_config, screen_config);

            // TODO! Проверка конфига!!

            stage_2_select_color_analyst(
                settings,
                shadow_root,
                geometry_config,
                screen_config,
                capture_thread,
                processor,
            );
        }
    }
}

/// 2-я ступень - выбор анализатора цвета в фрагменте
fn stage_2_select_color_analyst<Processor>(
    settings: Settings,
    shadow_root: FullConfigShadow,
    geometry_config: GeometryConfig,
    screen_config: ScreenConfig,
    capture_thread: &CaptureThread,
    processor: Processor,
) where
    Processor: ChunkProcessor,
{
    match settings.analytics_type {
        ColorAnalystType::ColorHistogram => {
            let analyst = ColorHistogram::new();

            stage_3_select_filter(
                settings,
                shadow_root,
                geometry_config,
                screen_config,
                capture_thread,
                processor,
                analyst,
            );
        }
    }
}

/// 3-я ступень - выбор фильтра для результирующего цвета
fn stage_3_select_filter<Processor, Analyst>(
    settings: Settings,
    shadow_root: FullConfigShadow,
    geometry_config: GeometryConfig,
    screen_config: ScreenConfig,
    capture_thread: &CaptureThread,
    processor: Processor,
    analyst: Analyst,
) where
    Processor: ChunkProcessor,
    Analyst: ColorAnalyst,
{
    match settings.filter_type {
        ColorFilterType::EmaFilter => {
            let filter = EmaFilter::new(geometry_config.calculate_leds_amount());

            stage_4_select_hardware_driver(
                settings,
                shadow_root,
                geometry_config,
                screen_config,
                capture_thread,
                processor,
                analyst,
                filter,
            );
        }
    }
}

/// 4-я ступень - выбор вывода на устройство
fn stage_4_select_hardware_driver<Processor, Analyst, Filter>(
    settings: Settings,
    shadow_root: FullConfigShadow,
    geometry_config: GeometryConfig,
    screen_config: ScreenConfig,
    capture_thread: &CaptureThread,
    processor: Processor,
    analyst: Analyst,
    filter: Filter,
) where
    Processor: ChunkProcessor,
    Analyst: ColorAnalyst,
    Filter: ColorFilter,
{
    let color_engine = ColorEngine::new(processor, analyst, filter, geometry_config.clone(), screen_config);

    match settings.hardware_output_type {
        HardwareOutputType::DebugDriver => {
            let hardware_output = DebugDriver::new(
                geometry_config.led_pos.horizontal_led_amount,
                geometry_config.led_pos.vertical_led_amount,
            );

            // Уже проверенный конфиг
            run_ambient_loop(color_engine, hardware_output, capture_thread);
        }

        HardwareOutputType::SerialDriver => {
            let Some(shadow) = shadow_root.serial_driver_config else {
                println!("Проверьте секцию [serial_driver_config]");
                return;
            };
            let serial_driver_config: SerialDriverConfig = shadow.into();

            // Проверка уже содержится в открытии порта
            let hardware_output = SerialDriver::new(serial_driver_config);

            run_ambient_loop(color_engine, hardware_output, capture_thread);
        }
    }
}

/// Запуск цикла обработки
fn run_ambient_loop<Processor, Analyst, Filter, Output>(
    mut color_engine: ColorEngine<Processor, Analyst, Filter>,
    mut hardware_output: Output,
    capture_thread: &CaptureThread,
) where
    Processor: ChunkProcessor,
    Analyst: ColorAnalyst,
    Filter: ColorFilter,
    Output: HardwareOutput,
{
    println!("Система запущена!");

    while KEEP_RUNNING.load(Ordering::Relaxed) {
        capture_thread.request_frame(|data| {
            // Теперь data — это безопасный &[u8]
            // Получаем указатель на вектор цветов
            let colors = color_engine.process_frame(data);

            hardware_output.send_colors(colors);
        });
    }

    println!("Цикл обработки завершен.");
}
