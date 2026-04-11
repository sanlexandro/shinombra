use algorithms::{
    analytics::registry::*,
    processing::{configs::*, registry::*},
    filters::registry::*,
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
    Router, body::Bytes, response::{IntoResponse, Redirect, Html, AppendHeaders}, routing::{get, post},
    extract::State,
    http::header::{SET_COOKIE, COOKIE},
};
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rand::Rng;

// ============================================================================
// МАКРОБРАБОТКА: Генерируем shadow версии структур для десериализации из TOML
// ============================================================================
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

// ============================================================================
// 2FA STATE: Состояние двухфакторной аутентификации
// ============================================================================
#[derive(Clone)]
struct Auth2FAState {
    sessions: Arc<Mutex<HashMap<String, (String, bool)>>>,
    disabled: bool,
    bot_token: Option<String>,
    user_id: Option<i64>,
}

impl Auth2FAState {
    fn new() -> Self {
        let disable_2fa = std::env::var("DISABLE_2FA").unwrap_or_default() == "true";
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").ok();
        let user_id = std::env::var("TELEGRAM_USER_ID").ok().and_then(|s| s.parse().ok());

        println!("[2FA] init: disabled={}, token_ok={}, user_id_ok={}", 
            disable_2fa, bot_token.is_some(), user_id.is_some());

        Auth2FAState {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            disabled: disable_2fa,
            bot_token,
            user_id,
        }
    }

    fn is_2fa_enabled(&self) -> bool {
        !self.disabled && self.bot_token.is_some() && self.user_id.is_some()
    }

    fn generate_session_id() -> String {
        let mut rng = rand::thread_rng();
        let random: u64 = rng.gen();
        format!("{:016x}", random)
    }

    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(100000..999999);
        format!("{}", code)
    }

    async fn send_code_to_telegram(&self, code: &str) -> bool {
        if let (Some(token), Some(user_id)) = (&self.bot_token, self.user_id) {
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let client = reqwest::Client::new();
            
            let message = format!("🔐 Код доступа к конфигуратору: {}", code);
            
            let params = serde_json::json!({
                "chat_id": user_id,
                "text": message
            });

            println!("[2FA] 📤 Отправляю код {} в Telegram (user_id={})", code, user_id);

            match client.post(&url).json(&params).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    println!("[2FA] ✓ Ответ Telegram: {}", status);
                    if !status.is_success() {
                        if let Ok(body) = resp.text().await {
                            eprintln!("[2FA] ✗ Ошибка API Telegram: {}", body);
                        }
                        return false;
                    }
                    true
                },
                Err(e) => {
                    eprintln!("[2FA] ✗ Ошибка HTTP при отправке в Telegram: {}", e);
                    false
                }
            }
        } else {
            eprintln!("[2FA] ✗ Telegram не настроен (token={}, user_id={:?})", 
                self.bot_token.is_some(), self.user_id);
            false
        }
    }
}

// ============================================================================
// ШАБЛОНЫ
// ============================================================================
#[derive(Template)]
#[template(path = "index.html")]
struct ConfigTemplate {
    settings: SettingsShadow,
    screen_config: ScreenConfigShadow,
    led_position_config: LedPositionConfigShadow,
    screen_reading_config: ScreenReadingConfigShadow,
    serial_driver_config: Option<SerialDriverConfigShadow>,
    checkerboard_config: Option<CheckerboardConfigShadow>,
}

#[derive(Template)]
#[template(path = "2fa.html")]
struct Auth2FATemplate;

// ============================================================================
// MAIN
// ============================================================================
#[tokio::main]
async fn main() {
    // Загружаем переменные окружения из .env файла
    dotenv::dotenv().ok();

    let auth_state = Auth2FAState::new();

    if auth_state.is_2fa_enabled() {
        println!("🔐 2FA включена (отправка кодов через Telegram)");
        println!("   Отключить: DISABLE_2FA=true");
    } else if auth_state.disabled {
        println!("✓ 2FA отключена (DISABLE_2FA=true)");
    } else {
        println!("⚠️  2FA недоступна (не установлены TELEGRAM_BOT_TOKEN и TELEGRAM_USER_ID)");
    }

    let app = Router::new()
        .route("/", get(show_home))
        .route("/verify-code", post(verify_code))
        .route("/config", get(show_config))
        .route("/update", post(update_config))
        .with_state(auth_state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("\n🌐 Конфигурация доступна по адресу http://{}\n", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================
fn get_config_path() -> String {
    "./cfg.toml".to_string()
}

fn extract_session_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get(COOKIE)?.to_str().ok()?
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            if cookie.starts_with("session=") {
                Some(cookie[8..].to_string())
            } else {
                None
            }
        })
}

// ============================================================================
// HANDLERS
// ============================================================================

async fn show_home(
    headers: axum::http::HeaderMap,
    State(auth): State<Auth2FAState>,
) -> impl IntoResponse {
    if !auth.is_2fa_enabled() {
        return show_config(headers, State(auth)).await.into_response();
    }

    // Проверяем есть ли cookie с сессией
    if let Some(session_id) = extract_session_cookie(&headers) {
        let should_show_config = {
            if let Ok(sessions) = auth.sessions.lock() {
                // Проверяем статус этой сессии
                if let Some((_, verified)) = sessions.get(&session_id) {
                    if *verified {
                        println!("[2FA-HOME] Session {} verified, показываю конфиг", session_id);
                        true  // Возвращаем true для show_config
                    } else {
                        println!("[2FA-HOME] Session {} exists but not verified, показываю форму", session_id);
                        return Html(Auth2FATemplate.render().unwrap()).into_response();
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };  // Заимствование опускается здесь
        
        if should_show_config {
            return show_config(headers, State(auth)).await.into_response();
        }
    }

    // Нет валидной сессии - нужно создать новую и отправить код
    println!("[2FA-HOME] Нет сессии, создаю новую...");
    
    let session_id = Auth2FAState::generate_session_id();
    let code = Auth2FAState::generate_code();
    
    println!("[2FA-CODE] ✅ Генерирую код: {}", code);
    
    if let Ok(mut sessions) = auth.sessions.lock() {
        sessions.insert(session_id.clone(), (code.clone(), false));
        println!("[2FA-CODE] ✅ Сохранил в sessions (всего: {})", sessions.len());
    } else {
        eprintln!("[2FA-CODE] ✗ Не могу заблокировать sessions!");
    }

    // Отправляем код в Telegram асинхронно
    tokio::spawn({
        let auth = auth.clone();
        let code = code.clone();
        async move {
            println!("[2FA-CODE] 🚀 Запускаю отправку в фоне. Код: {}", code);
            let result = auth.send_code_to_telegram(&code).await;
            println!("[2FA-CODE] 📨 Результат отправки в Telegram: {}", result);
        }
    });

    println!("[2FA-CODE] Показываю форму ввода кода");
    Html(Auth2FATemplate.render().unwrap()).into_response()
}

async fn verify_code(
    State(auth): State<Auth2FAState>,
    body: Bytes,
) -> impl IntoResponse {
    let body_str = String::from_utf8_lossy(&body);
    println!("[2FA-VERIFY] body={}", body_str);
    
    let entered_code = body_str
        .split('&')
        .find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            if key == "code" {
                urlencoding::decode(value).ok()
            } else {
                None
            }
        })
        .unwrap_or_default()
        .to_string();

    println!("[2FA-VERIFY] entered_code='{}', length={}", entered_code, entered_code.len());

    if let Ok(mut sessions) = auth.sessions.lock() {
        println!("[2FA-VERIFY] active_sessions={}", sessions.len());
        // Найдем сессию с этим кодом и обновим её
        for (sid, (code, _verified)) in sessions.iter_mut() {
            println!("[2FA-VERIFY]   checking code='{}' (len={})", code, code.len());
            if code == &entered_code {
                println!("[2FA-VERIFY] ✓✓✓ КОД СОВПАДАЕТ!");
                *_verified = true;
                let session_id = sid.clone();
                let cookie = format!("session={}; Path=/; HttpOnly; Max-Age=3600", session_id);
                println!("[2FA-VERIFY] Setting cookie: {}", cookie);
                return (AppendHeaders([(SET_COOKIE, cookie)]), Redirect::to("/")).into_response();
            }
        }
        println!("[2FA-VERIFY] ✗ Код не совпадает ни с одним из активных");
    } else {
        println!("[2FA-VERIFY] ✗ Не могу заблокировать sessions Mutex");
    }

    println!("[2FA-VERIFY] Возвращаю 302 redirect на /");
    Redirect::to("/").into_response()
}

async fn show_config(
    headers: axum::http::HeaderMap,
    State(auth): State<Auth2FAState>,
) -> Html<String> {
    if auth.is_2fa_enabled() {
        if let Some(session_id) = extract_session_cookie(&headers) {
            if let Ok(sessions) = auth.sessions.lock() {
                if sessions.get(&session_id).is_none() {
                    let session_id = Auth2FAState::generate_session_id();
                    let code = Auth2FAState::generate_code();
                    
                    if let Ok(mut sessions) = auth.sessions.lock() {
                        sessions.insert(session_id.clone(), (code.clone(), false));
                    }

                    tokio::spawn({
                        let auth = auth.clone();
                        let code = code.clone();
                        async move {
                            println!("[2FA-CODE] 🚀 Запускаю отправку в фоне. Код: {}", code);
                            let result = auth.send_code_to_telegram(&code).await;
                            println!("[2FA-CODE] Результат отправки: {}", result);
                        }
                    });

                    println!("[2FA-CODE] Новая 2FA сессия для 1-го случая. Код: {} (сохранён в sessions)", code);
                    return Html(Auth2FATemplate.render().unwrap());
                } else if let Some((_, true)) = sessions.get(&session_id) {
                    // Авторизован, продолжаем
                } else {
                    // Session не верифицирован
                    return Html(Auth2FATemplate.render().unwrap());
                }
            }
        } else {
            // Нет cookie - генерируем новую сессию
            let session_id = Auth2FAState::generate_session_id();
            let code = Auth2FAState::generate_code();
            
            if let Ok(mut sessions) = auth.sessions.lock() {
                sessions.insert(session_id.clone(), (code.clone(), false));
            }

            tokio::spawn({
                let auth = auth.clone();
                let code = code.clone();
                async move {
                    println!("[2FA-CODE] 🚀 Запускаю отправку в фоне (нет cookie). Код: {}", code);
                    let result = auth.send_code_to_telegram(&code).await;
                    println!("[2FA-CODE] Результат отправки: {}", result);
                }
            });

            println!("[2FA-CODE] Новая 2FA сессия без cookie. Код: {} (сохранён в sessions)", code);
            return Html(Auth2FATemplate.render().unwrap());
        }
    }

    let config_path = get_config_path();
    let toml_str = std::fs::read_to_string(&config_path).unwrap_or_default();
    let shadow_root: FullConfigShadow = match toml::from_str(&toml_str) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("[CONFIG] ✗ Ошибка парсинга TOML: {}", e);
            eprintln!("[CONFIG] Content:\n{}", &toml_str[..std::cmp::min(500, toml_str.len())]);
            FullConfigShadow::default()
        }
    };
    
    println!("[CONFIG] Loaded checkerboard_config: {:?}", shadow_root.checkerboard_config.is_some());
    println!("[CONFIG] Loaded serial_driver_config: {:?}", shadow_root.serial_driver_config.is_some());

    // Если checkerboard_config не загружен, создаем дефолтный
    let checkerboard_config = shadow_root.checkerboard_config.or_else(|| {
        println!("[CONFIG] Creating default checkerboard_config");
        Some(CheckerboardConfigShadow {
            config: ChunkConfigShadow {
                width: PixelsShadow(640),
                height: PixelsShadow(480),
            },
            pixel_step: 1,
            row_stride: 1,
        })
    });

    let template = ConfigTemplate {
        settings: shadow_root.settings.unwrap_or_default(),
        screen_config: shadow_root.screen_config.unwrap_or_default(),
        led_position_config: shadow_root.led_position_config.unwrap_or_default(),
        screen_reading_config: shadow_root.screen_reading_config.unwrap_or_default(),
        serial_driver_config: shadow_root.serial_driver_config,
        checkerboard_config,
    };

    Html(template.render().unwrap())
}

async fn update_config(
    headers: axum::http::HeaderMap,
    State(auth): State<Auth2FAState>,
    body: Bytes,
) -> impl IntoResponse {
    if auth.is_2fa_enabled() {
        if let Some(session_id) = extract_session_cookie(&headers) {
            if let Ok(sessions) = auth.sessions.lock() {
                if let Some((_, true)) = sessions.get(&session_id) {
                    // Авторизован, продолжаем
                } else {
                    return Redirect::to("/").into_response();
                }
            } else {
                return Redirect::to("/").into_response();
            }
        } else {
            return Redirect::to("/").into_response();
        }
    }

    let body_str = String::from_utf8_lossy(&body);
    println!("\n\n===== FORM DATA RECEIVED =====");
    println!("{}", body_str);
    println!("================================\n");

    let config_path = get_config_path();
    let old_toml = std::fs::read_to_string(&config_path).unwrap_or_default();
    let mut final_root: FullConfigShadow = toml::from_str(&old_toml).unwrap_or_default();

    let mut json_value = match serde_json::to_value(&final_root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("✗ Conversion error: {}", e);
            return Redirect::to("/config").into_response();
        }
    };

    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let key = urlencoding::decode(key).unwrap_or_default().to_string();
            let value = urlencoding::decode(value).unwrap_or_default().to_string();
            
            if value.is_empty() {
                continue;
            }
            
            println!("Processing: {} = {}", key, value);
            set_nested_json_value(&mut json_value, &key, &value);
        }
    }

    match serde_json::from_value::<FullConfigShadow>(json_value) {
        Ok(updated) => final_root = updated,
        Err(e) => {
            eprintln!("✗ Deserialization error: {}", e);
            return Redirect::to("/config").into_response();
        }
    }

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

    Redirect::to("/config").into_response()
}

fn set_nested_json_value(obj: &mut serde_json::Value, path: &str, value: &str) {
    let keys: Vec<&str> = path.split('.').collect();
    if keys.is_empty() {
        return;
    }

    let mut current = obj;
    
    for (i, &key) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
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
            if !current[key].is_object() {
                current[key] = serde_json::json!({});
            }
            current = &mut current[key];
        }
    }
}
