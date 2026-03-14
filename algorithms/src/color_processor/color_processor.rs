//! Алгоритм обработки массива цвета

#[derive(PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct ScreenConfig {
    pub frame_width: usize,  // ширина кадра
    pub frame_height: usize, // высота кадра
}

pub struct CheckerboardConfig {
    pub chunk_width: usize,  // ширина фрагмента
    pub chunk_height: usize, // высота фрагмента

    pub pixel_step: usize, // шаг чтения пикселей строки
    pub row_stride: usize, // шаг чтения строк
}

struct HSVPixel {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

struct RGBPixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Copy, Clone)]
struct ColorBin {
    pub weight: u64,
}

impl ColorBin {
    pub fn new() -> Self {
        Self { weight: 0 }
    }
}

/// Ф-я преобразования BGR (*Blue Green Red*) формата в HSV (*Hue Saturation
/// Value*)
fn convert_bgr_to_hsv(bgr: &[u8], hsv: &mut HSVPixel) {
    let max = bgr[0].max(bgr[1]).max(bgr[2]) as f32;
    let min = bgr[0].min(bgr[1]).min(bgr[2]) as f32;

    let delta = max - min;

    hsv.value = max;

    hsv.saturation = if max == 0.0 { 0.0 } else { delta / max };

    hsv.hue = if delta == 0.0 {
        0.0
    } else if max == bgr[2] as f32 {
        // Приводим К f32 ДО вычитания
        60.0 * (((bgr[1] as f32 - bgr[0] as f32) / delta) % 6.0)
    } else if max == bgr[1] as f32 {
        60.0 * (((bgr[0] as f32 - bgr[2] as f32) / delta) + 2.0)
    } else {
        60.0 * (((bgr[2] as f32 - bgr[1] as f32) / delta) + 4.0)
    };

    if hsv.hue < 0.0 {
        hsv.hue += 360.0;
    }
}

fn process_votes(bins: &mut [ColorBin], hsv_pixel: &HSVPixel) {
    let weight = (hsv_pixel.value * hsv_pixel.saturation) as u64;

    if weight < 10 {
        bins[36].weight += weight + 1; // чтобы чёрный всегда голосовал
        return;
    }

    let bin_idx = (hsv_pixel.hue / 10.0) as usize;
    bins[bin_idx].weight += weight;
}

fn determining_winner_bin(bins: &[ColorBin]) -> HSVPixel {
    let (winner_idx, winner_bin) = bins
        .iter()
        .enumerate()
        .max_by_key(|(_, bin)| bin.weight)
        .expect("bins must not be empty");

    // Если голосов вообще нет или победил "черный" (36-й индекс)
    if winner_idx == 36 || winner_bin.weight == 0 {
        return HSVPixel {
            hue: 0.0,
            saturation: 0.0,
            value: 0.0,
        };
    }

    // Рассчитываем Hue как центр сектора
    // Т.к. индекс 0 — это 0-10°, центр будет 5°
    let hue = (winner_idx as f32 * 10.0) + 5.0;

    HSVPixel {
        hue,
        saturation: 1.0, // Пока константа
        value: 255.0,    // Пока константа (макс яркость)
    }
}

/// Ф-я перевода HSV формата в RGB
fn convert_hsv_to_rgb(hsv_pixel: &HSVPixel) -> (RGBPixel) {
    let chroma = hsv_pixel.value * hsv_pixel.saturation;
    let x = chroma * (1.0 - ((hsv_pixel.hue / 60.0) % 2.0 - 1.0).abs());
    let m = hsv_pixel.value - chroma;

    let (r_prime, g_prime, b_prime) = if hsv_pixel.hue < 60.0 {
        (chroma, x, 0.0)
    } else if hsv_pixel.hue < 120.0 {
        (x, chroma, 0.0)
    } else if hsv_pixel.hue < 180.0 {
        (0.0, chroma, x)
    } else if hsv_pixel.hue < 240.0 {
        (0.0, x, chroma)
    } else if hsv_pixel.hue < 300.0 {
        (x, 0.0, chroma)
    } else {
        (chroma, 0.0, x)
    };

    // Просто прибавляем m и кастуем, так как значения уже в 0..255
    return RGBPixel {
        red: (r_prime + m).round() as u8,
        green: (g_prime + m).round() as u8,
        blue: (b_prime + m).round() as u8,
    };
}

/// Ф-я вывода цвета в консоль для отладки
fn print_debug_color(rgb_pixel: RGBPixel) {
    // \x1b[48;2;R;G;Bm — цвет фона (TrueColor)
    // \x1b[0m — сброс
    println!(
        "\x1b[48;2;{};{};{}m      \x1b[0m Winner (RGB: {}, {}, {})",
        rgb_pixel.red,
        rgb_pixel.green,
        rgb_pixel.blue,
        rgb_pixel.red,
        rgb_pixel.green,
        rgb_pixel.blue
    );
}

/// Ф-я обработки одного фрагмента (жёсткий шахматный порядок)
pub fn process_checkerboard_chunk(
    byte_frame: &[u8],
    screen_config: &ScreenConfig,
    alg_config: &CheckerboardConfig,
    chunk_start_index: usize,
    chunk_orientation: Orientation,
) {
    // Определяем реальный размер строки в байтах
    let row_width = screen_config.frame_width * 4;

    // В зависимости от положения кадра определяем его ширину и высоту
    let chunk_width = if chunk_orientation == Orientation::Horizontal {
        alg_config.chunk_width
    } else {
        alg_config.chunk_height
    };

    let chunk_height = if chunk_orientation == Orientation::Horizontal {
        alg_config.chunk_height
    } else {
        alg_config.chunk_width
    };

    let row_stride = if chunk_orientation == Orientation::Horizontal {
        alg_config.row_stride
    } else {
        alg_config.pixel_step
    };

    let pixel_step = if chunk_orientation == Orientation::Horizontal {
        alg_config.pixel_step
    } else {
        alg_config.row_stride
    };

    // Массив для сбора голосов за сектора
    let mut bins = [ColorBin::new(); 36 + 1];

    // Цикл построчного чтения (читаем каждую `row_stride` строку)
    for row_idx in (0..chunk_height).step_by(row_stride) {
        // Вычисляем стартовый индекс строки (стартовый индекс + (ширина экрана
        // row_strideкол-во строк))
        let row_start_index = chunk_start_index + row_width * row_idx;

        // В зависимости от чётности строки начинаем строку либо с самого начала
        // либо со сдвигом на половину шага чтения строки
        let start = if (row_idx / row_stride) % 2 == 1 {
            row_start_index + (pixel_step / 2) * 4
        } else {
            row_start_index
        };
        let end = row_start_index + chunk_width * 4;

        // fixme добавить логику смещения строк по чётности

        // Делаем срез строки (от стартового индекса строки, до (него + ширина
        // фрагмента))
        let row_bytes = &byte_frame[start..end];
        let pixels = row_bytes.chunks_exact(4).step_by(pixel_step);

        // --- Отрабатываем пиксели --- //
        let mut hsv_pixel = HSVPixel {
            hue: 0.0,
            saturation: 0.0,
            value: 0.0,
        };

        // Итерируемся по срезу (читаем каждый `pixel_step` пиксель)
        for brg_pixel in pixels {
            // Переводим в HSV формат
            convert_bgr_to_hsv(brg_pixel, &mut hsv_pixel);

            // Проводим голосование
            process_votes(&mut bins, &hsv_pixel);
        }
    }

    let winner_hsv_pixel = determining_winner_bin(&bins);

    print_debug_color(convert_hsv_to_rgb(&winner_hsv_pixel));
}

pub fn alg(byte_frame: &[u8], screen_config: &ScreenConfig, alg_config: &CheckerboardConfig) {
    process_checkerboard_chunk(
        byte_frame,
        screen_config,
        alg_config,
        0,
        Orientation::Horizontal,
    );
}
