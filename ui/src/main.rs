use std::{path::PathBuf, process::exit, sync::Arc};

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use algorithms::{
    analytics::{configs::*, registry::*},
    filters::{configs::*, registry::*},
    processing::{
        configs::*,
        processors::{configs::*, registry::*},
        registry::*,
    },
};
use ambient_core::config::*;
use common::{names::*, paths::*, units::*};
use config_gen::{__private::*, *};
use hardware_output::{
    ddp::config::*, registry::*, shinombra_serial::config::*, wled_drgb::config::*,
};
use logger::*;
use std::process::Command;
use ui_gen::{__private::serde_json, add_js, generate_ui, Renderable};

// Подключаем все тени
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
    "./hardware_output/src/shinombra_serial/config/config.rs",
    "./hardware_output/src/wled_drgb/config/config.rs",
    "./hardware_output/src/ddp/config/config.rs",
    "./infra/logger/src/registry.rs",
);
// Генерируем ui
generate_ui!(
    "./core/src/config.rs" => {
        Settings => {
            chunk_processor_type: Registry {"./algorithms/src/processing/processors/registry.rs" => ChunkProcessorType },
            analytics_type: Registry {"./algorithms/src/analytics/registry.rs" => ColorAnalystType },
            hardware_output_type: Registry {"./hardware_output/src/registry.rs" => HardwareOutputType },
            filter_chain: WrapperVec( Registry {"./algorithms/src/filters/registry.rs" => ColorFilterType} ),
            max_fps: Option( NumericField {60} ),
        },
        Flags => {},
        Paths => {},
        DaemonSettings => {
            log_level: Registry {"./infra/logger/src/registry.rs" => LogLevel },
            pipewire_conversion: BoolField {},
            save_token: BoolField {},
        },
        Manifest => {},
    },
    "./algorithms/src/processing/configs/configs.rs" => {
        ScreenConfig => {
            frame_width_mm: Wrapper(Millimeters, NumericField {0}),
            frame_height_mm: Wrapper(Millimeters, NumericField {0})
        },
        LedPositionConfig => {
            gap: Wrapper(Millimeters, NumericField {0}),
            led_length: Wrapper(Millimeters, NumericField {0}),
            vertical_led_amount: NumericField{1},
            horizontal_led_amount: NumericField{1}
        },
        FrameConnectionConfig => {
            start_from: Registry {"./algorithms/src/processing/registry.rs" => FrameElement},
            direction: Registry {"./algorithms/src/processing/registry.rs" => ClockDirection},
        },
        ScreenReadingConfig => {
            deep_in: Wrapper(Millimeters, NumericField {0}),
            deep_out: Wrapper(Millimeters, NumericField {0})
        },
        GeometryConfig => {},
        GeometryPlusScreenConfig => {},
    },
    "./algorithms/src/processing/processors/configs/configs.rs" => {
        ChunkConfig => {},
        @show_if(Settings.chunk_processor_type == "Checkerboard")
        CheckerboardConfig => {
            pixel_step: NumericField{1},
            row_stride: NumericField{1}
        },
        @show_if(Settings.chunk_processor_type == "DynamicCheckerboard")
        DynamicCheckerboardConfig => {
            pixel_step: NumericField{1},
            row_stride: NumericField{1},

            column_crawl: NumericField{1},
            row_crawl: NumericField{1},
        },
        ChunkTask => {},
    },
    "./algorithms/src/analytics/configs/configs.rs" => {
        @show_if(Settings.analytics_type == "Histogram")
        HistogramConfig => {
            precision_level: SliderField { 0.0, 20.0, 1 }
        },
        @show_if(Settings.analytics_type == "DebugRGB")
        DebugRGBConfig => {
            red: SliderField { 0, 255, 1 },
            green: SliderField { 0, 255, 1 },
            blue: SliderField { 0, 255, 1 }
        }
    },
    "./algorithms/src/filters/configs/configs.rs" => {
        @show_if(Settings.filter_chain.includes("Gamma"))
        GammaConfig => {
            gamma: SliderField { 0.01, 4.0, 0.01}
        },
        @show_if(Settings.filter_chain.includes("Ema"))
        EmaConfig => {
            alpha: SliderField { 0.01, 1.0, 0.01 }
        },
        @show_if(Settings.filter_chain.includes("BlackThreshold"))
        BlackThresholdConfig => {
            threshold: SliderField {0.0, 100.0, 0.1 },
            fade_range: SliderField {0.0, 100.0, 0.1 },
            falloff_exponent: SliderField {0.0, 4.0, 0.1 }
        },
        @show_if(Settings.filter_chain.includes("SaturationBoost"))
        SaturationBoostConfig => {
            floor: SliderField {0.0, 100.0, 0.1 },
            boost_exponent: SliderField {0.0, 4.0, 0.1 },
        },
        @show_if(Settings.filter_chain.includes("WhiteBalance"))
        WhiteBalanceConfig => {
            kelvins: SliderField {0, 20000, 100}
        },
        @show_if(Settings.filter_chain.includes("ChannelGain"))
        ChannelGainConfig => {
            k_red: SliderField {0.0, 1.0, 0.01},
            k_green: SliderField {0.0, 1.0, 0.01},
            k_blue: SliderField {0.0, 1.0, 0.01},
        },
        @show_if(Settings.filter_chain.includes("FlashGuard"))
        FlashGuardConfig => {
            alpha: SliderField { 0.01, 1.0, 0.01 },
            sensitivity: SliderField {0.0, 100.0, 0.1 },
        }
    },
    "./hardware_output/src/shinombra_serial/config/config.rs" => {
        @show_if(Settings.hardware_output_type == "ShinombraSerial")
        ShinombraSerialConfig => {
            port_path: TextField {"port path"},
            baud_rate: NumericField {9600}
        }
    },
    "./hardware_output/src/wled_drgb/config/config.rs" => {
        @show_if(Settings.hardware_output_type == "WledDrgb")
        WledDrgbConfig => {
            ip: TextField {"ip or DNS-name"},
            port: Option ( NumericField {} ),
            timeout: Option ( NumericField {} ),
        }
    },
    "./hardware_output/src/ddp/config/config.rs" => {
        @show_if(Settings.hardware_output_type == "Ddp")
        DdpConfig => {
            ip: TextField {"ip or DNS-name"},
            port: Option ( NumericField {} ),
            timeout: Option ( NumericField {} ),
            mtu: Option ( NumericField {} ),
        }
    },
);

/// Состояние приложения
#[derive(Clone)]
pub struct AppState {
    pub config_path: Arc<RwLock<Option<PathBuf>>>,
}

#[derive(Deserialize)]
struct ConfigPathForm {
    config_path: String,
}

async fn render_start_page() -> Html<String> {
    let html = format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>{} v{}</title>
            <link rel="stylesheet" href="/style.css">
            <link rel="icon" href="/image.jpg" type="image/jpeg">
        </head>
        <body>
            <main style="max-width: 720px; margin: 64px auto; padding: 24px;">
                <h1>Choose config file</h1>
                <p>Start the app by selecting the path to your config file first.</p>
                <form action="/config-path" method="post" style="display: flex; gap: 12px; align-items: end; flex-wrap: wrap;">
                    <label style="flex: 1; min-width: 280px; display: flex; flex-direction: column; gap: 8px;">
                        <span>Config path</span>
                        <input type="text" name="config_path" placeholder="/path/to/config.toml" required style="width: 100%;">
                    </label>
                    <button type="submit">Open Config</button>
                </form>
            </main>
        </body>
        </html>"#,
        APP_NAME,
        env!("CARGO_PKG_VERSION")
    );

    Html(html)
}

/// Основная страница
async fn show_index(State(state): State<AppState>) -> axum::response::Response {
    let config_path = state.config_path.read().await.clone();

    let Some(config_path) = config_path else {
        return render_start_page().await.into_response();
    };

    // Подгружаем данные из файла
    let toml_str = std::fs::read_to_string(&config_path).unwrap_or_default();
    let shadow_root: FullConfigShadow = toml::from_str(&toml_str).unwrap_or_default();

    let html = format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>{} v{}</title>
            <link rel="stylesheet" href="/style.css">
            <link rel="icon" href="/image.jpg" type="image/jpeg">
        </head>
        <body>
            <form action="/save" method="post">
                {}
                <div class="actions-group">
                    <button type="button" class="btn-secondary" onclick="setServiceEnv()">Set Simple Mode in Service</button>
                    <button type="button" class="btn-secondary" onclick="restartService()">Restart Service</button>
                    <button type="submit">Save Config</button>
                </div>
            </form>
            <script>
                {}

                async function setServiceEnv() {{
                    try {{
                        const res = await fetch('/set-env', {{ method: 'POST' }});
                        if (res.ok) {{
                            alert('Successfully set --config in service.env!');
                        }} else {{
                            const err = await res.text();
                            alert('Error: ' + err);
                        }}
                    }} catch (e) {{
                        alert('Network error: ' + e);
                    }}
                }}

                async function restartService() {{
                    try {{
                        const res = await fetch('/restart-service', {{ method: 'POST' }});
                        if (res.ok) {{
                            alert('Shinombra service restarted successfully!');
                        }} else {{
                            const err = await res.text();
                            alert('Failed to restart service: ' + err);
                        }}
                    }} catch (e) {{
                        alert('Network error: ' + e);
                    }}
                }}
            </script>
        </body>
        </html>"#,
        APP_NAME,
        env!("CARGO_PKG_VERSION"),
        render_full_html(&shadow_root),
        add_js()
    );

    Html(html).into_response()
}

/// Обработчик сохранения
async fn save_config(
    State(state): State<AppState>,
    axum::Json(raw_json): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let config_path = state.config_path.read().await.clone();

    let Some(config_path) = config_path else {
        return axum::http::StatusCode::BAD_REQUEST;
    };

    // Читаем текущее состояние файла
    let toml_str = std::fs::read_to_string(&config_path).unwrap_or_default();
    let mut current_config: FullConfigShadow = toml::from_str(&toml_str).unwrap_or_default();

    // Накладываем патч из пришедшего JSON непосредственно на существующий конфиг
    current_config.apply_json_patch(raw_json);

    // Читаем исходный файл в таблицу
    let mut file_toml: toml::Table = toml::from_str(&toml_str).unwrap_or_default();

    // Сериализуем обновленную структуру во временное TOML-значение
    if let Ok(toml::Value::Table(patched_table)) = toml::Value::try_from(current_config) {
        // Просто обновляем значения в file_toml
        for (key, value) in patched_table {
            file_toml.insert(key, value);
        }
    }

    // Сохраняем результат
    if let Ok(new_toml_str) = toml::to_string_pretty(&file_toml) {
        let _ = std::fs::write(&config_path, new_toml_str);
    }

    axum::http::StatusCode::OK
}

/// Выбор пути к конфигу
async fn set_config_path(
    State(state): State<AppState>,
    Form(form): Form<ConfigPathForm>,
) -> impl IntoResponse {
    let config_path = expand_tilde(PathBuf::from(form.config_path));

    {
        let mut stored_path = state.config_path.write().await;
        *stored_path = Some(config_path);
    }

    axum::response::Redirect::to("/")
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

/// Записывает `--config /path/to/config` в `service.env`
async fn set_env_config(State(state): State<AppState>) -> impl IntoResponse {
    let config_path = state.config_path.read().await.clone();

    let Some(config_path) = config_path else {
        return (axum::http::StatusCode::BAD_REQUEST, "Config path not set").into_response();
    };

    let home = std::env::var_os("HOME").map(PathBuf::from);
    let env_dir = home
        .map(|h| h.join(".config/shinombra"))
        .unwrap_or_else(|| PathBuf::from("./.config/shinombra"));

    if let Err(e) = std::fs::create_dir_all(&env_dir) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create config dir: {}", e),
        )
            .into_response();
    }

    let env_file_path = env_dir.join("service.env");
    let env_content = format!("SHINOMBRA_ARGS=\"--config {}\"\n", config_path.display());

    if let Err(e) = std::fs::write(&env_file_path, env_content) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write service.env: {}", e),
        )
            .into_response();
    }

    (axum::http::StatusCode::OK, "Successfully set env config").into_response()
}

/// Перезапускает пользовательский systemd сервис shinombra
async fn restart_service() -> impl IntoResponse {
    let output = Command::new("systemctl")
        .args(["--user", "restart", "shinombra"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            (axum::http::StatusCode::OK, "Service restarted successfully").into_response()
        }
        Ok(out) => {
            let err_msg = String::from_utf8_lossy(&out.stderr);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to restart service: {}", err_msg),
            )
                .into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Systemctl command failed: {}", e),
        )
            .into_response(),
    }
}

/// Поддержка изображений
// async fn image_imge() -> impl IntoResponse {
//     let image = include_bytes!("../assets/image.jpg");
//     (
//         axum::http::StatusCode::OK,
//         [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
//         image.to_vec(),
//     )
// }

#[tokio::main]
async fn main() {
    // Обработка аргументов
    let mut args = std::env::args().skip(1);

    // Значения по умолчанию
    let mut port: u32 = 3000;
    let mut config_path = PathBuf::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-p" | "--port" => {
                if let Some(next_arg) = args.next() {
                    port = match next_arg.parse() {
                        Ok(p) => p,
                        Err(e) => {
                            println!("Can not parse {} as port: {}", next_arg, e);
                            exit(1);
                        }
                    }
                } else {
                    println!("Can not find port value after {}.\nUse {} <port>", arg, arg);
                    exit(1);
                }
            }
            "--config" => {
                if let Some(next_arg) = args.next() {
                    config_path = expand_tilde(PathBuf::from(next_arg));
                } else {
                    println!(
                        "Can not find config path after {}.\nUse {} </path/to/config>",
                        arg, arg
                    );
                    exit(1);
                }
            }
            "--help" => {
                println!(include_str!("../../docs/help_ui.txt"));
                exit(0);
            }
            unknown => {
                println!("Unknown arg {} will be skipped", unknown);
            }
        }
    }

    let state = AppState {
        config_path: Arc::new(RwLock::new(
            (!config_path.as_os_str().is_empty()).then_some(config_path),
        )),
    };

    let app = Router::new()
        .route("/", get(show_index))
        .route("/config-path", axum::routing::post(set_config_path))
        .route("/style.css", get(style_css))
        .route("/save", axum::routing::post(save_config))
        .route("/set-env", axum::routing::post(set_env_config))
        .route("/restart-service", axum::routing::post(restart_service))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("Server running on http://localhost:{}", port);
    axum::serve(listener, app).await.unwrap();
}
