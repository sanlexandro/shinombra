//! Подключение ф-ий из C

use std::fmt::Display;
use std::os::raw::{c_char, c_void};
use std::sync::Arc;

use common::core::controller::CoreController;

/// Структура для обмена данными с потоком захвата
///
/// **Поля:**
/// `screen_width`: [u32] - ширина экрана в пикселях
/// `screen_height`: [u32] - высота экрана в пикселях
/// `video_format`: [u32] - формат видео (конвертируется в [SpaVideoFormat])
#[repr(C)]
pub struct CaptureConfig {
    pub screen_width: u32,  // Заполняет C (PipeWire)
    pub screen_height: u32, // Заполняет C (PipeWire)
    pub video_format: u32,  // Заполняет C (PipeWire)
}

/// Реализация методов для [CaptureConfig]
impl CaptureConfig {
    /// Конструктор
    ///
    /// Создаёт конфиг с нулевыми значениями и ложным флагом
    pub fn new() -> Self {
        return Self {
            screen_width: 0,
            screen_height: 0,
            video_format: 0,
        };
    }
}

/// Состояния потока захвата
#[repr(C)]
pub enum CaptureEvent {
    Ready = 0,
    Error = 1,
    Stopped = 2,
    Reconnecting = 3,
}

/// Данные для инициализации
///
/// **Поля:**
/// - `apply_conversion`: [bool] - разрешение применения конвертации форматов
/// - `save_token`: [bool] - разрешение на сохранение токена
/// - `token`: *const [c_char] - сам токен
#[repr(C)]
pub struct InitializingData {
    pub apply_conversion: bool,
    pub save_token: bool,
    pub token: *const c_char,
}

/// Тип ф-ии обратного вызова
pub type CaptureEventCallback =
    extern "C" fn(user_data: *mut c_void, event: CaptureEvent, message: *const c_char);

// Используем ф-ии из `C-worker`
extern "C" {
    /// Инициализация захвата экрана
    pub fn screen_capture_init(
        config: *mut CaptureConfig,
        callback: CaptureEventCallback,
        user_data: *const c_void,
        init_data: InitializingData,
    ) -> *mut c_void;

    /// Запуск захвата экрана
    pub fn screen_capture_run(ctx: *mut c_void);

    /// Остановка потока захвата экрана
    pub fn screen_capture_stop(ctx: *mut c_void);

    /// Получение текущей конфигурации захвата экрана
    pub fn get_capture_config(ctx: *mut c_void) -> *mut CaptureConfig;

    // Получение токена для восстановления сессии захвата экрана
    pub fn get_restore_token(ctx: *mut c_void) -> *const c_char;

    /// Получение указателя DMA
    pub fn wait_for_frame(ctx: *mut c_void) -> *mut u8;

    /// Освобождение кадра DMA
    pub fn release_frame(ctx: *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn release_user_data(user_data: *mut c_void) {
    if !user_data.is_null() {
        // Восстанавливаем Arc и позволяем ему выйти из области видимости,
        // что уменьшит счетчик ссылок и удалит объект, если ссылок больше нет.
        let _ = Arc::from_raw(user_data as *const CoreController);
    }
}

/// Тип видео-формата
///
/// Скопировано из `spa/param/video/raw.h` `spa_video_format`
#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(u32)]
pub enum SpaVideoFormat {
    UNKNOWN = 0,
    ENCODED = 1,

    I420 = 2,
    YV12 = 3,
    YUY2 = 4,
    UYVY = 5,
    AYUV = 6,
    RGBx = 7,
    BGRx = 8,
    xRGB = 9,
    xBGR = 10,
    RGBA = 11,
    BGRA = 12,
    /// native
    ARGB = 13,
    ABGR = 14,
    RGB = 15,
    BGR = 16,
    Y41B = 17,
    Y42B = 18,
    YVYU = 19,
    Y444 = 20,
    v210 = 21,
    v216 = 22,
    NV12 = 23,
    NV21 = 24,
    GRAY8 = 25,
    GRAY16_BE = 26,
    GRAY16_LE = 27,
    v308 = 28,
    RGB16 = 29,
    BGR16 = 30,
    RGB15 = 31,
    BGR15 = 32,
    UYVP = 33,
    A420 = 34,
    RGB8P = 35,
    YUV9 = 36,
    YVU9 = 37,
    IYU1 = 38,
    ARGB64 = 39,
    AYUV64 = 40,
    r210 = 41,
    I420_10BE = 42,
    I420_10LE = 43,
    I422_10BE = 44,
    I422_10LE = 45,
    Y444_10BE = 46,
    Y444_10LE = 47,
    GBR = 48,
    GBR_10BE = 49,
    GBR_10LE = 50,
    NV16 = 51,
    NV24 = 52,
    NV12_64Z32 = 53,
    A420_10BE = 54,
    A420_10LE = 55,
    A422_10BE = 56,
    A422_10LE = 57,
    A444_10BE = 58,
    A444_10LE = 59,
    NV61 = 60,
    P010_10BE = 61,
    P010_10LE = 62,
    IYU2 = 63,
    VYUY = 64,
    GBRA = 65,
    GBRA_10BE = 66,
    GBRA_10LE = 67,
    GBR_12BE = 68,
    GBR_12LE = 69,
    GBRA_12BE = 70,
    GBRA_12LE = 71,
    I420_12BE = 72,
    I420_12LE = 73,
    I422_12BE = 74,
    I422_12LE = 75,
    Y444_12BE = 76,
    Y444_12LE = 77,

    RGBA_F16 = 78,
    RGBA_F32 = 79,

    /**< 32-bit x:R:G:B 2:10:10:10 little endian */
    xRGB_210LE = 80,
    /**< 32-bit x:B:G:R 2:10:10:10 little endian */
    xBGR_210LE = 81,
    /**< 32-bit R:G:B:x 10:10:10:2 little endian */
    RGBx_102LE = 82,
    /**< 32-bit B:G:R:x 10:10:10:2 little endian */
    BGRx_102LE = 83,
    /**< 32-bit A:R:G:B 2:10:10:10 little endian */
    ARGB_210LE = 84,
    /**< 32-bit A:B:G:R 2:10:10:10 little endian */
    ABGR_210LE = 85,
    /**< 32-bit R:G:B:A 10:10:10:2 little endian */
    RGBA_102LE = 86,
    /**< 32-bit B:G:R:A 10:10:10:2 little endian */
    BGRA_102LE = 87,
    /* Aliases */
    // DSP_F32 = 79, // RGBA_F32
}

impl TryFrom<u32> for SpaVideoFormat {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 87 {
            return Err(value);
        }

        return Ok(unsafe { std::mem::transmute(value) });
    }
}

impl Display for SpaVideoFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
