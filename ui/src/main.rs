use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use config_gen::{__private::*, *};
use ui_gen::{add_js, generate_ui, Renderable};

use algorithms::{
    analytics::registry::*,
    filters::registry::*,
    processing::{
        configs::*,
        processors::{
            configs::{CheckerboardConfig, ChunkConfig, ChunkTask},
            registry::*,
        },
    },
    units::*,
};
use ambient_core::config::Settings;
use hardware_output::{registry::*, serial::config::SerialDriverConfig};

// Подключаем все тени
include_shadow_all!(
    "./algorithms/src/units.rs",
    "./algorithms/src/processing/configs.rs",
    "./algorithms/src/processing/processors/registry.rs",
    "./algorithms/src/processing/processors/configs.rs",
    "./algorithms/src/analytics/registry.rs",
    "./algorithms/src/filters/registry.rs",
    "./hardware_output/src/registry.rs",
    "./core/src/config.rs",
    "./hardware_output/src/serial/config.rs"
);
// Генерируем ui
generate_ui!(
    "./core/src/config.rs" => {
        Settings => {
            chunk_processor_type: Registry {"./algorithms/src/processing/processors/registry.rs" => ChunkProcessorType },
            analytics_type: Registry {"./algorithms/src/analytics/registry.rs" => ColorAnalystType },
            hardware_output_type: Registry {"./hardware_output/src/registry.rs" => HardwareOutputType },
            filter_chain: WrapperVec( Registry {"./algorithms/src/filters/registry.rs" => ColorFilterType} ),
        },
    },
    "./algorithms/src/processing/configs.rs" => {
        ScreenConfig => {
            frame_width_mm: Wrapper(Millimeters, NumericField {0}),
            frame_height_mm: Wrapper(Millimeters, NumericField {0})
        },
        LedPositionConfig => {
            gap: Wrapper(Millimeters, NumericField {0}),
            vertical_offset: Wrapper(Millimeters, NumericField {0}),
            horizontal_offset: Wrapper(Millimeters, NumericField {0}),
            led_length: Wrapper(Millimeters, NumericField {0}),
            vertical_led_amount: NumericField{1},
            horizontal_led_amount: NumericField{1}
        },
        ScreenReadingConfig => {
            deep_in: Wrapper(Millimeters, NumericField {0}),
            deep_out: Wrapper(Millimeters, NumericField {0})
        },
        GeometryConfig => {},
        ChunkConfig => {},
        CheckerboardConfig => {
            pixel_step: NumericField{1},
            row_stride: NumericField{1}
        },
        ChunkTask => {},
    },
    "./hardware_output/src/serial/config.rs" => {
        @show_if(Settings.hardware_output_type == "SerialDriver")
        SerialDriverConfig => {
            port_path: TextField {"port path"},
            baud_rate: NumericField {9600}
        }
    },
);

/// Основная страница
async fn show_index() -> impl IntoResponse {
    // Подгружаем данные из файла
    let toml_str = std::fs::read_to_string("./cfg.toml").expect("no_file");
    let shadow_root: FullConfigShadow = toml::from_str(&toml_str).unwrap_or_default();

    let html = format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Config UI</title>
            <link rel="stylesheet" href="/style.css">
        </head>
        <body>
            <form action="/save" method="post">
                {}
                <button type="submit">Save Config</button>
            </form>
            <script>
                {}
            </script>
        </body>
        </html>"#,
        render_full_html(&shadow_root),
        add_js()
    );

    // Отправляем сформированную страницу
    Html(html).into_response()
}

/// Обработчик сохранения
async fn save_config(axum::Json(raw_json): axum::Json<serde_json::Value>) -> impl IntoResponse {
    // Читаем текущее состояние файла
    let toml_str = std::fs::read_to_string("./cfg.toml").unwrap_or_default();
    let mut current_config: FullConfigShadow = toml::from_str(&toml_str).unwrap_or_default();

    // Создаем "патч" из пришедшего JSON
    let patch = FullConfigShadow::from_json(raw_json);

    // Накладываем патч на текущий конфиг
    current_config.apply_patch(patch);

    // Сохраняем результат
    if let Ok(toml_str) = toml::to_string_pretty(&current_config) {
        let _ = std::fs::write("./cfg.toml", toml_str);
    }

    axum::http::StatusCode::OK
}

/// Поддержка стилей
async fn style_css() -> impl IntoResponse {
    let css = include_str!("../assets/style.css");
    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/css")],
        css,
    )
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(show_index))
        .route("/style.css", get(style_css))
        .route("/save", axum::routing::post(save_config));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
