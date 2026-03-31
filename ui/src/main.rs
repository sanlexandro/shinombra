use algorithms::{
    analytics::registry::*,
    processing::{configs::*, registry::*},
    units::*,
};
use ambient_core::config::Settings;
use config_gen::{__private::*, *};
use hardware_output::{
    registry::*,
    serial::config::SerialDriverConfig,
};

use askama::Template;
use axum::{
    Router, body::Bytes, response::{IntoResponse, Redirect, Html}, routing::{get, post}
};
use std::fs;
use std::net::SocketAddr;

// ============================================================================
// МАКРОБРАБОТКА: Генерируем shadow версии структур для десериализации из TOML
// ============================================================================
// include_shadow_all! макрос читает исходные Rust файлы с конфигурационными
// структурами и создает "shadow" версии с поддержкой Serde для парсинга TOML.
// Исходные структуры остаются без изменений, как требовалось.
include_shadow_all!(
    "./algorithms/src/units.rs",
    "./algorithms/src/processing/configs.rs",
    "./algorithms/src/processing/registry.rs",
    "./algorithms/src/analytics/registry.rs",
    "./hardware_output/src/registry.rs",
    "./core/src/config.rs",
    "./hardware_output/src/serial/config.rs"
);

// ============================================================================
// ДАННЫЕ ДЛЯ ШАБЛОНА: Основная структура для передачи информации в HTML
// ============================================================================
// Содержит все конфигурационные данные, которые нужны для отображения
// интерфейса и визуализации настроек
#[derive(Template)]
#[template(path = "index.html")]
struct ConfigTemplate {
    // === Критические конфиги (обязательны всегда) ===
    settings: SettingsShadow,
    screen_config: ScreenConfigShadow,
    led_position_config: LedPositionConfigShadow,
    screen_reading_config: ScreenReadingConfigShadow,

    // === Условные конфиги (зависят от Settings) ===
    serial_driver_config: Option<SerialDriverConfigShadow>,
    checkerboard_config: Option<CheckerboardConfigShadow>,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(show_form))
        .route("/update", post(update_config));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Конфигурация доступна по адресу http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ============================================================================
// ФУНКЦИЯ: Получить путь к cfg.toml
// ============================================================================
fn get_config_path() -> String {
    "./cfg.toml".to_string()
}

// ============================================================================
// ОБРАБОТЧИК GET: Отображение страницы с текущей конфигурацией
// ============================================================================
async fn show_form() -> Html<String> {
    // Читаем существующий TOML файл
    let config_path = get_config_path();
    let toml_str = std::fs::read_to_string(&config_path).unwrap_or_default();
    let shadow_root: FullConfigShadow = toml::from_str(&toml_str).unwrap_or_default();

    // Извлекаем все необходимые конфиги из сохраненной конфигурации
    // Критические конфиги получают значения по умолчанию, если не заданы
    let settings = shadow_root
        .settings
        .unwrap_or_default();

    let screen_config = shadow_root
        .screen_config
        .unwrap_or_default();

    let led_position_config = shadow_root
        .led_position_config
        .unwrap_or_default();

    let screen_reading_config = shadow_root
        .screen_reading_config
        .unwrap_or_default();

    // Условные конфиги: сохраняем как есть (Option<T>)
    let serial_driver_config = shadow_root.serial_driver_config;
    let checkerboard_config = shadow_root.checkerboard_config;

    // Формируем данные для шаблона
    let template = ConfigTemplate {
        settings,
        screen_config,
        led_position_config,
        screen_reading_config,
        serial_driver_config,
        checkerboard_config,
    };

    // Возвращаем HTML страницу с правильным Content-Type
    Html(template.render().unwrap())
}

// ============================================================================
// ОБРАБОТЧИК POST: Обновление конфигурации из отправленной формы
// ============================================================================
async fn update_config(body: Bytes) -> impl IntoResponse {
    let body_str = String::from_utf8_lossy(&body);
    println!("\n\n===== FORM DATA RECEIVED =====");
    println!("{}", body_str);
    println!("================================\n");

    // Читаем текущий конфиг 
    let config_path = get_config_path();
    let old_toml = std::fs::read_to_string(&config_path).unwrap_or_default();
    let mut final_root: FullConfigShadow = toml::from_str(&old_toml).unwrap_or_default();

    // Преобразуем текущую конфигурацию в JSON для удобного обновления
    let mut json_value = match serde_json::to_value(&final_root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("✗ Conversion error: {}", e);
            return Redirect::to("/");
        }
    };

    // Парсим форму и обновляем значения в JSON по пути
    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let key = urlencoding::decode(key).unwrap_or_default().to_string();
            let value = urlencoding::decode(value).unwrap_or_default().to_string();
            
            // Пропускаем пустые значения
            if value.is_empty() {
                continue;
            }
            
            println!("Processing: {} = {}", key, value);
            
            // Устанавливаем значение в JSON по пути (например: "screen_config.frame_width_mm")
            set_nested_json_value(&mut json_value, &key, &value);
        }
    }

    // Конвертируем JSON обратно в структуру
    match serde_json::from_value::<FullConfigShadow>(json_value) {
        Ok(updated) => final_root = updated,
        Err(e) => {
            eprintln!("✗ Deserialization error: {}", e);
            return Redirect::to("/");
        }
    }

    // Сохраняем в правильном TOML формате
    println!("\n=== SAVING CONFIG ===");
    match toml::to_string_pretty(&final_root) {
        Ok(output_toml) => {
            println!("TOML to save:\n{}\n", output_toml);
            match fs::write(&config_path, &output_toml) {
                Ok(_) => println!("✓✓✓ Configuration SAVED to: {}\n", config_path),
                Err(e) => eprintln!("✗ Write error: {}\n", e),
            }
        },
        Err(e) => eprintln!("✗ Serialization error: {}\n", e),
    }

    Redirect::to("/")
}

// ============================================================================
// ВСПОМОГАТЕЛЬНАЯ ФУНКЦИЯ: Установка значения в nested JSON по пути
// ============================================================================
// Пример: set_nested_json_value(&mut obj, "screen_config.frame_width_mm", "600")
// Автоматически парсит числа, строки, булевы значения.
// Полностью универсальная — работает с любой структурой!
fn set_nested_json_value(obj: &mut serde_json::Value, path: &str, value: &str) {
    let keys: Vec<&str> = path.split('.').collect();
    if keys.is_empty() {
        return;
    }

    let mut current = obj;
    
    for (i, &key) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
            // Последний элемент пути — устанавливаем значение
            // Пытаемся распарсить как число, булево, иначе строка
            if let Ok(num) = value.parse::<u32>() {
                current[key] = serde_json::json!(num);
            } else if let Ok(num) = value.parse::<usize>() {
                current[key] = serde_json::json!(num);
            } else if let Ok(b) = value.parse::<bool>() {
                current[key] = serde_json::json!(b);
            } else {
                current[key] = serde_json::json!(value.to_string());
            }
        } else {
            // Промежуточный ключ — убеждаемся, что это объект
            if !current[key].is_object() {
                current[key] = serde_json::json!({});
            }
            current = &mut current[key];
        }
    }
}